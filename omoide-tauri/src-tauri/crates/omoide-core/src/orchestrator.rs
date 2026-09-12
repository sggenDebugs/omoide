use std::path::{Path, PathBuf};

use omoide_crypto::kdf::{derive_entry_key, derive_master_key};
use omoide_crypto::MasterKey;
use omoide_env::{REPROMPT_MAX_RETRIES, REPROMPT_TIMEOUT_SECS};
use omoide_format::schema::{Entry, SrsState, VaultFile};

use crate::error::AuthError;
use crate::srs::{get_current_time_secs, is_rehearsal_due, next_interval};
use crate::vault::{open, seal};

// ─── Internal session types ────────────────────────────────────────────────

/// Data held while the vault is fully unlocked.
/// Dropping this struct zeroizes the MasterKey via ZeroizeOnDrop.
struct UnlockedSession {
    /// Held for re-encryption when the vault needs to be re-sealed.
    /// Zeroized on drop.
    #[allow(dead_code)]
    master_key: MasterKey,
    /// Decrypted entries, held in memory for the duration of the session.
    entries: Vec<Entry>,
    /// Snapshot of the SRS state from the vault header at unlock time.
    srs_snapshot: SrsState,
    /// A copy of the full vault file needed to re-seal on reprompt success.
    vault_file: VaultFile,
}

/// Data held while a reprompt challenge is in progress.
/// Note: MasterKey is absent. The user must re-derive it.
struct RepromptContext {
    /// Entries stay resident so the session can continue after a successful reprompt.
    /// Zeroized on drop on Entry fields.
    #[allow(dead_code)]
    entries: Vec<Entry>,
    /// SRS snapshot at the moment the reprompt was triggered.
    srs_snapshot: SrsState,
    /// The full vault file is kept so we can re-seal with updated SRS state.
    vault_file: VaultFile,
    /// Unix timestamp when this reprompt window started.
    reprompt_started_at_secs: u64,
    /// How many seconds the user has to respond before the vault auto-locks.
    timeout_secs: u64,
    /// How many attempts the user still has.
    retries_remaining: u8,
}

// ─── State Machine ─────────────────────────────────────────────────────────

enum VaultState {
    Locked,
    Unlocked(UnlockedSession),
    AwaitingReprompt(RepromptContext),
}

// ─── Auth Orchestrator ───────────────────────────────────────────────────

/// Auth Orchestrator — the central state machine for vault session lifecycle.
///
/// ### State transitions
///
/// ```text
/// Initial          ─► Locked
/// Locked           ─► Unlocked          : unlock() with correct password
/// Locked           ─► Locked            : unlock() with wrong password
/// Unlocked         ─► AwaitingReprompt  : check_reprompt() when SRS due
/// Unlocked         ─► Locked            : lock() / process exit
/// AwaitingReprompt ─► Unlocked          : submit_reprompt() correct password
/// AwaitingReprompt ─► AwaitingReprompt  : submit_reprompt() wrong, retries left
/// AwaitingReprompt ─► Locked            : submit_reprompt() retries exhausted
/// AwaitingReprompt ─► Locked            : check_timeout() deadline exceeded
/// ```
///
/// ### Security properties
/// - `MasterKey` is **dropped** on the `Unlocked → AwaitingReprompt` transition.
///   The user must re-type their password.
/// - All entries and the `MasterKey` are zeroized on any `→ Locked` transition via
///   `ZeroizeOnDrop` on the dropped session structs.
/// - `entries()` returns `None` in any state other than `Unlocked`.
pub struct AuthOrchestrator {
    state: VaultState,
    vault_path: PathBuf,
    reprompt_timeout_secs: u64,
    reprompt_max_retries: u8,
}

impl AuthOrchestrator {
    /// Create a new orchestrator for the given vault path.
    /// Starts in `Locked` state. Uses default timeout/retry constants.
    pub fn new(vault_path: PathBuf) -> Self {
        Self {
            state: VaultState::Locked,
            vault_path,
            reprompt_timeout_secs: REPROMPT_TIMEOUT_SECS,
            reprompt_max_retries: REPROMPT_MAX_RETRIES,
        }
    }

    /// Override reprompt timeout (seconds). Useful for tests.
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.reprompt_timeout_secs = secs;
        self
    }

    /// Override max retry count. For testing purposes only.
    pub fn with_max_retries(mut self, retries: u8) -> Self {
        self.reprompt_max_retries = retries;
        self
    }

    /// Attempt to unlock the vault with the provided password.
    ///
    /// On success, transitions `Locked → Unlocked`.
    /// Returns `AlreadyUnlocked` if already in `Unlocked` or `AwaitingReprompt`.
    pub fn unlock(&mut self, password: &str) -> Result<(), AuthError> {
        if !matches!(self.state, VaultState::Locked) {
            return Err(AuthError::AlreadyUnlocked);
        }

        let vault_file = open(&self.vault_path)?;
        let master_key = derive_master_key(
            password.as_bytes(),
            &vault_file.header.salt,
            &vault_file.header.kdf_params,
        )?;

        let entries = decrypt_all_entries(&master_key, &vault_file)?;
        let srs_snapshot = vault_file.header.srs_state.clone();

        self.state = VaultState::Unlocked(UnlockedSession {
            master_key,
            entries,
            srs_snapshot,
            vault_file,
        });

        Ok(())
    }

    /// Check whether the SRS interval has elapsed.
    ///
    /// If due, transitions into `Unlocked → AwaitingReprompt` (dropping the `MasterKey`).
    /// Returns `true` when a reprompt is now required.
    /// Returns `false` when not in `Unlocked` state or when the interval has not elapsed.
    pub fn check_reprompt(&mut self, now_secs: u64) -> bool {
        let due = match &self.state {
            VaultState::Unlocked(session) => is_rehearsal_due(&session.srs_snapshot, now_secs),
            _ => return false,
        };

        if due {
            // Take ownership of the session, then build the reprompt context.
            // The UnlockedSession (and its MasterKey) are moved out and dropped
            // at the end of this block — ZeroizeOnDrop wipes the key bytes.
            if let VaultState::Unlocked(session) = std::mem::replace(
                &mut self.state,
                VaultState::Locked, // lock vault first to avoid holding the key for too long
            ) {
                self.state = VaultState::AwaitingReprompt(RepromptContext {
                    entries: session.entries,
                    srs_snapshot: session.srs_snapshot,
                    vault_file: session.vault_file,
                    reprompt_started_at_secs: now_secs,
                    timeout_secs: self.reprompt_timeout_secs,
                    retries_remaining: self.reprompt_max_retries,
                });
            }
        }
        due
    }

    /// Submit a password attempt for an active reprompt challenge.
    ///
    /// - Correct password + retries available:  
    ///   `AwaitingReprompt → Unlocked`, SRS state updated and vault re-sealed.
    /// - Wrong password + retries remaining:  
    ///   stays `AwaitingReprompt`, returns `RepromptFailed { retries_left }`.
    /// - Wrong password + no retries remaining:  
    ///   `AwaitingReprompt → Locked`, SRS penalty applied, vault re-sealed, secrets zeroized.
    ///   Returns `AllRetriesExhausted` if all retry attempts are exhausted.
    pub fn submit_reprompt(&mut self, password: &str) -> Result<(), AuthError> {
        if !matches!(self.state, VaultState::AwaitingReprompt(_)) {
            return Err(AuthError::NotAwaitingReprompt);
        }

        // Temporarily replace state with Locked to take the custom context out.
        let custom = match std::mem::replace(&mut self.state, VaultState::Locked) {
            VaultState::AwaitingReprompt(custom) => custom,
            _ => unreachable!(),
        };

        // Try to re-derive the master key and verify via decryption probe.
        let verify = derive_master_key(
            password.as_bytes(),
            &custom.vault_file.header.salt,
            &custom.vault_file.header.kdf_params,
        )
        .and_then(|master_key| {
            decrypt_all_entries(&master_key, &custom.vault_file)
                .map(|entries| (master_key, entries))
        });

        match verify {
            Ok((master_key, entries)) => {
                let now = get_current_time_secs();
                let new_interval = next_interval(custom.srs_snapshot.current_interval_hours, true);

                let mut updated_vault = custom.vault_file;
                updated_vault.header.srs_state.last_rehearsal = now;
                updated_vault.header.srs_state.current_interval_hours = new_interval;
                updated_vault.header.srs_state.consecutive_failures = 0;

                // Atomically save vault changes.
                seal(&updated_vault, &self.vault_path)?;

                let new_srs = updated_vault.header.srs_state.clone();
                self.state = VaultState::Unlocked(UnlockedSession {
                    master_key,
                    entries,
                    srs_snapshot: new_srs,
                    vault_file: updated_vault,
                });

                Ok(())
            }
            Err(_) => {
                let retries_left = custom.retries_remaining.saturating_sub(1);

                if retries_left > 0 {
                    // Entries remain in memory. Stay in AwaitingReprompt.
                    self.state = VaultState::AwaitingReprompt(RepromptContext {
                        retries_remaining: retries_left,
                        ..custom
                    });
                    return Err(AuthError::RepromptFailed { retries_left });
                }

                // All retries exhausted. Apply penalty, save changes, then lock.
                let now = get_current_time_secs();
                let new_interval = next_interval(custom.srs_snapshot.current_interval_hours, false);

                let mut updated_vault = custom.vault_file;
                updated_vault.header.srs_state.last_rehearsal = now;
                updated_vault.header.srs_state.current_interval_hours = new_interval;
                updated_vault.header.srs_state.consecutive_failures =
                    custom.srs_snapshot.consecutive_failures.saturating_add(1);

                // Save SRS penalty to vault file. We log but ignore failures here to ensure
                // the transition to 'Locked' is never blocked by I/O errors.
                if let Err(e) = seal(&updated_vault, &self.vault_path) {
                    eprintln!("[orchestrator] WARNING: failed to persist SRS penalty: {e}");
                }

                // State remains Locked (set by the mem::replace above).
                // Custom (including entries) is dropped here, zeroized and dropped.
                Err(AuthError::AllRetriesExhausted)
            }
        }
    }

    /// Check whether the reprompt window has timed out.
    ///
    /// If timed out, transitions `AwaitingReprompt → Locked` with SRS penalty applied.
    /// Returns `true` when the timeout has fired (regardless of prior state).
    pub fn check_timeout(&mut self, now_secs: u64) -> bool {
        let timed_out = match &self.state {
            VaultState::AwaitingReprompt(custom) => {
                now_secs >= custom.reprompt_started_at_secs + custom.timeout_secs
            }
            _ => return false,
        };

        if timed_out {
            let custom = match std::mem::replace(&mut self.state, VaultState::Locked) {
                VaultState::AwaitingReprompt(custom) => custom,
                _ => unreachable!(),
            };

            // Apply SRS timeout penalty.
            let new_interval = next_interval(custom.srs_snapshot.current_interval_hours, false);

            let mut updated_vault = custom.vault_file;
            updated_vault.header.srs_state.last_rehearsal = now_secs;
            updated_vault.header.srs_state.current_interval_hours = new_interval;
            updated_vault.header.srs_state.consecutive_failures =
                custom.srs_snapshot.consecutive_failures.saturating_add(1);

            if let Err(e) = seal(&updated_vault, &self.vault_path) {
                eprintln!("[orchestrator] WARNING: failed to persist SRS timeout penalty: {e}");
            }

            // custom (entries) is dropped here — ZeroizeOnDrop wipes them.
        }

        timed_out
    }

    /// Explicitly lock the vault, zeroizing all secrets.
    ///
    /// Safe to call in any state. No-op if already `Locked`.
    pub fn lock(&mut self) {
        // mem::replace drops the old state, triggering ZeroizeOnDrop on entries
        // and MasterKey inside UnlockedSession / RepromptContext.
        self.state = VaultState::Locked;
    }

    /// Returns decrypted entries, or `None` if not in `Unlocked` state.
    pub fn entries(&self) -> Option<&[Entry]> {
        match &self.state {
            VaultState::Unlocked(session) => Some(&session.entries),
            _ => None,
        }
    }

    /// A short, non-secret state label for logging.
    pub fn state_label(&self) -> &'static str {
        match &self.state {
            VaultState::Locked => "Locked",
            VaultState::Unlocked(_) => "Unlocked",
            VaultState::AwaitingReprompt(_) => "AwaitingReprompt",
        }
    }

    /// Returns the vault file path.
    pub fn vault_path(&self) -> &Path {
        &self.vault_path
    }
}

/// Decrypt all entries in the vault using the given master key.
/// Returns a plain `CryptoError` (wrapped as `AuthError::Crypto`) on any failure.
fn decrypt_all_entries(
    master_key: &MasterKey,
    vault_file: &VaultFile,
) -> Result<Vec<Entry>, omoide_crypto::CryptoError> {
    let aad = &vault_file.header.header_aad;
    let mut entries = Vec::with_capacity(vault_file.entries.len());

    for enc in &vault_file.entries {
        let entry_key = derive_entry_key(master_key, &enc.id, b"entry-enc")?;
        let plaintext = omoide_crypto::decrypt_entry(&entry_key, &enc.nonce, aad, &enc.ciphertext)?;
        let entry: Entry = ciborium::de::from_reader(plaintext.as_slice())
            .map_err(|_| omoide_crypto::CryptoError::DecryptionFailed)?;
        entries.push(entry);
    }

    Ok(entries)
}

// ─── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use omoide_crypto::kdf::KdfParams;
    use omoide_crypto::{aead::encrypt_entry, kdf::derive_entry_key};
    use omoide_format::schema::{EncryptedEntry, VaultFile, VaultHeader};
    use tempfile::tempdir;

    // Fast KDF params so tests don't take 300ms each.
    fn test_kdf() -> KdfParams {
        KdfParams {
            memory_cost: 1024,
            time_cost: 1,
            parallelism_cost: 1,
        }
    }

    fn make_test_vault(path: &Path, password: &str) {
        use omoide_crypto::kdf::derive_master_key;
        use rand::{rngs::SysRng, TryRng};

        let params = test_kdf();
        let mut salt = [0u8; 32];
        let mut aad = [0u8; 32];
        SysRng.try_fill_bytes(&mut salt).unwrap();
        SysRng.try_fill_bytes(&mut aad).unwrap();

        let master_key = derive_master_key(password.as_bytes(), &salt, &params).unwrap();

        // Build one entry.
        let entry = Entry {
            title: "Test".into(),
            username: "user".into(),
            password: "secret".into(),
            url: "https://example.com".into(),
            notes: "".into(),
            created: 0,
            updated: 0,
        };
        let mut cbor = Vec::new();
        ciborium::ser::into_writer(&entry, &mut cbor).unwrap();

        let mut id = [0u8; 16];
        let mut nonce = [0u8; 12];
        SysRng.try_fill_bytes(&mut id).unwrap();
        SysRng.try_fill_bytes(&mut nonce).unwrap();

        let entry_key = derive_entry_key(&master_key, &id, b"entry-enc").unwrap();
        let ciphertext = encrypt_entry(&entry_key, &nonce, &aad, &cbor).unwrap();

        let vault = VaultFile {
            header: VaultHeader {
                kdf_params: params,
                salt,
                header_aad: aad,
                srs_state: SrsState::default(),
            },
            entries: vec![EncryptedEntry {
                id,
                nonce,
                ciphertext,
            }],
        };
        seal(&vault, path).unwrap();
    }

    #[test]
    fn test_initial_state_is_locked() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("v.omoide");
        make_test_vault(&path, "pw");
        let orch = AuthOrchestrator::new(path);
        assert_eq!(orch.state_label(), "Locked");
    }

    #[test]
    fn test_state_label_correctness() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("v.omoide");
        make_test_vault(&path, "pw");

        // It is assumed that all the reprompt attempts has been consumed,
        // as well as the srs interval has elapsed.
        // NOTE: We use .with_max_retries(3) for testing purposes only.
        let mut orch = AuthOrchestrator::new(path.clone()).with_max_retries(3);
        assert_eq!(orch.state_label(), "Locked");

        orch.unlock("pw").unwrap();
        assert_eq!(orch.state_label(), "Unlocked");

        // Force reprompt by using a past timestamp for last_rehearsal
        let far_future = u64::MAX / 2;
        orch.check_reprompt(far_future);
        assert_eq!(orch.state_label(), "AwaitingReprompt");
    }

    #[test]
    fn test_unlock_success() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("v.omoide");
        make_test_vault(&path, "correct");
        let mut orch = AuthOrchestrator::new(path);
        assert!(orch.unlock("correct").is_ok());
        assert_eq!(orch.state_label(), "Unlocked");
        assert!(orch.entries().is_some());
    }

    #[test]
    fn test_unlock_wrong_password() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("v.omoide");
        make_test_vault(&path, "correct");
        let mut orch = AuthOrchestrator::new(path);
        let err = orch.unlock("wrong").unwrap_err();
        assert!(matches!(err, AuthError::Crypto(_)));
        assert_eq!(orch.state_label(), "Locked");
        assert!(orch.entries().is_none());
    }

    #[test]
    fn test_unlock_when_already_unlocked() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("v.omoide");
        make_test_vault(&path, "pw");
        let mut orch = AuthOrchestrator::new(path);
        orch.unlock("pw").unwrap();
        let err = orch.unlock("pw").unwrap_err();
        assert!(matches!(err, AuthError::AlreadyUnlocked));
    }

    #[test]
    fn test_check_reprompt_not_due() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("v.omoide");
        make_test_vault(&path, "pw");
        let mut orch = AuthOrchestrator::new(path);
        orch.unlock("pw").unwrap();
        // now = 1 — interval is 12h so not due
        let triggered = orch.check_reprompt(1);
        assert!(!triggered);
        assert_eq!(orch.state_label(), "Unlocked");
    }

    #[test]
    fn test_check_reprompt_due_transitions_state() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("v.omoide");
        make_test_vault(&path, "pw");
        let mut orch = AuthOrchestrator::new(path);
        orch.unlock("pw").unwrap();
        // Far future — rehearsal definitely due
        let triggered = orch.check_reprompt(u64::MAX / 2);
        assert!(triggered);
        assert_eq!(orch.state_label(), "AwaitingReprompt");
        // MasterKey dropped — entries still resident
        assert!(orch.entries().is_none()); // entries() only available in Unlocked
    }

    #[test]
    fn test_reprompt_success_transitions_to_unlocked() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("v.omoide");
        make_test_vault(&path, "pw");
        let mut orch = AuthOrchestrator::new(path);
        orch.unlock("pw").unwrap();
        orch.check_reprompt(u64::MAX / 2);
        assert_eq!(orch.state_label(), "AwaitingReprompt");

        orch.submit_reprompt("pw").unwrap();
        assert_eq!(orch.state_label(), "Unlocked");
        assert!(orch.entries().is_some());
    }

    #[test]
    fn test_reprompt_single_failure_stays_awaiting_with_retries_left() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("v.omoide");
        make_test_vault(&path, "pw");
        let mut orch = AuthOrchestrator::new(path).with_max_retries(3);
        orch.unlock("pw").unwrap();
        orch.check_reprompt(u64::MAX / 2);

        let err = orch.submit_reprompt("wrong").unwrap_err();
        assert!(matches!(err, AuthError::RepromptFailed { retries_left: 2 }));
        assert_eq!(orch.state_label(), "AwaitingReprompt");
    }

    #[test]
    fn test_reprompt_all_retries_exhausted_locks() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("v.omoide");
        make_test_vault(&path, "pw");
        let mut orch = AuthOrchestrator::new(path).with_max_retries(3);
        orch.unlock("pw").unwrap();
        orch.check_reprompt(u64::MAX / 2);

        // Three wrong attempts
        assert!(matches!(
            orch.submit_reprompt("wrong").unwrap_err(),
            AuthError::RepromptFailed { retries_left: 2 }
        ));
        assert!(matches!(
            orch.submit_reprompt("wrong").unwrap_err(),
            AuthError::RepromptFailed { retries_left: 1 }
        ));
        let final_err = orch.submit_reprompt("wrong").unwrap_err();
        assert!(matches!(final_err, AuthError::AllRetriesExhausted));
        assert_eq!(orch.state_label(), "Locked");
        assert!(orch.entries().is_none());
    }

    #[test]
    fn test_reprompt_timeout_locks() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("v.omoide");
        make_test_vault(&path, "pw");
        let mut orch = AuthOrchestrator::new(path).with_timeout(10);
        orch.unlock("pw").unwrap();

        // SRS default: last_rehearsal=0, interval=12h=43200s.
        // Use a timestamp well past the interval to guarantee the reprompt fires.
        let trigger_now = 100_000u64;
        let triggered = orch.check_reprompt(trigger_now); // reprompt_started_at = 100_000
        assert!(
            triggered,
            "reprompt should fire when now is well past due time"
        );
        assert_eq!(orch.state_label(), "AwaitingReprompt");

        // Before timeout (100_000 + 10 = 100_010)
        assert!(!orch.check_timeout(trigger_now + 5));
        assert_eq!(orch.state_label(), "AwaitingReprompt");

        // At/after timeout
        assert!(orch.check_timeout(trigger_now + 10));
        assert_eq!(orch.state_label(), "Locked");
        assert!(orch.entries().is_none());
    }

    #[test]
    fn test_explicit_lock_from_unlocked() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("v.omoide");
        make_test_vault(&path, "pw");
        let mut orch = AuthOrchestrator::new(path);
        orch.unlock("pw").unwrap();
        assert!(orch.entries().is_some());

        orch.lock();
        assert_eq!(orch.state_label(), "Locked");
        assert!(orch.entries().is_none());
    }

    #[test]
    fn test_srs_state_persisted_on_reprompt_success() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("v.omoide");
        make_test_vault(&path, "pw");

        let mut orch = AuthOrchestrator::new(path.clone());
        orch.unlock("pw").unwrap();
        let original_interval = {
            let vault = open(&path).unwrap();
            vault.header.srs_state.current_interval_hours
        };

        orch.check_reprompt(u64::MAX / 2);
        orch.submit_reprompt("pw").unwrap();

        let vault = open(&path).unwrap();
        assert!(vault.header.srs_state.current_interval_hours > original_interval);
        assert_eq!(vault.header.srs_state.consecutive_failures, 0);
    }

    #[test]
    fn test_srs_state_persisted_on_reprompt_failure_after_retries_exhausted() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("v.omoide");
        make_test_vault(&path, "pw");

        let mut orch = AuthOrchestrator::new(path.clone()).with_max_retries(1);
        orch.unlock("pw").unwrap();
        orch.check_reprompt(u64::MAX / 2);
        let _ = orch.submit_reprompt("wrong"); // exhausts 1 retry

        let vault = open(&path).unwrap();
        assert_eq!(vault.header.srs_state.consecutive_failures, 1);
    }
}
