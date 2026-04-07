use crate::error::CryptoError;
use crate::types::{MasterKey, EntryKey};
use argon2::{Argon2, Algorithm, Version, Params};
use hkdf::Hkdf;
use sha2::Sha256;
use omoide_env::*;

/// KDF parameters frozen at vault creation time.
/// Stored in vault header plaintext so the vault can be reopened
/// without knowing params in advance.
/// DO NOT change defaults without running `omoide bench-kdf` and updating ADR-001.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KdfParams {
    pub memory_cost:      u32, // memory in KiB — default: 19456 (19 MiB)
    pub time_cost:        u32, // iterations   — default: 2
    pub parallelism_cost: u32, // parallelism  — default: 1
}

// Set KDF Parameters to default values
impl Default for KdfParams {
    fn default() -> Self {
        Self {
            memory_cost:      KDFPARAMS_MEM_COST,
            time_cost:        KDFPARAMS_TIME_COST,
            parallelism_cost: KDFPARAMS_PARALLEL_COST,
        }
    }
}

/// Derives a MasterKey from a password and salt using Argon2id.
///
/// ### Arguments
/// - `password` — raw UTF-8 bytes of the master password
/// - `salt`     — 32 random bytes stored in vault header, never reused
/// - `params`   — KDF parameters frozen at vault creation time
pub fn derive_master_key(
    password: &[u8],
    salt: &[u8; KDF_SALT_SIZE],
    params: &KdfParams,
) -> Result<MasterKey, CryptoError> {
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(
            params.memory_cost,
            params.time_cost,
            params.parallelism_cost,
            ARGON_OUTPUT_SIZE
        )?,
    );
    let mut key = MasterKey::new_zeroed();
    argon2.hash_password_into(password, salt, key.expose_mut())?;
    Ok(key)
}


/// Derives a 32-byte per-entry key from the MasterKey via HKDF-SHA256.
///
/// ### Arguments
/// - `master`   — the vault master key (never used directly for encryption)
/// - `entry_id` — UUID bytes of the entry (16 bytes) — used as HKDF salt
/// - `info`     — purpose tag of key usage:
///  `b"entry-enc"` for vault entries
///  `b"seed-wrap"` for BIP39 recovery seed (reserved, future)
pub fn derive_entry_key(
    master: &MasterKey,
    entry_id: &[u8; ENTRY_ID_SIZE],
    info: &[u8],
) -> Result<EntryKey, CryptoError> {
    let hashed_key = Hkdf::<Sha256>::new(
        Some(entry_id),
        master.expose_secret()
    );
    let mut key = EntryKey::new_zeroed();
    hashed_key.expand(info, key.expose_mut())
        .map_err(|_| CryptoError::HkdfExpand)?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    // fixed salt for deterministic tests — never use a fixed salt in production
    const TEST_SALT: &[u8; 32] = b"omoide_test_salt_123456789_test1";
    const TEST_PASSWORD: &[u8] = b"test-password-1234";

    // use minimal params in tests — default params take ~300ms per call
    fn fast_params() -> KdfParams {
        KdfParams {
            memory_cost:      1024, // 1024 KiB — fast enough for tests
            time_cost:        1,
            parallelism_cost: 1,
        }
    }

    #[test]
    fn derive_master_key_succeeds() {
        // T2: KDF produces a non-zero key
        let key = derive_master_key(TEST_PASSWORD, TEST_SALT, &fast_params())
            .expect("KDF failed");
        // key must not be all zeros — Argon2id must have done work
        assert_ne!(key.expose_secret(), &[0u8; 32]);
    }

    #[test]
    fn derive_master_key_is_deterministic() {
        // same password + salt + params must always produce the same key
        // this is required for vault reopening to work
        let key1 = derive_master_key(TEST_PASSWORD, TEST_SALT, &fast_params()).unwrap();
        let key2 = derive_master_key(TEST_PASSWORD, TEST_SALT, &fast_params()).unwrap();
        assert_eq!(key1.expose_secret(), key2.expose_secret());
    }

    #[test]
    fn different_passwords_produce_different_keys() {
        // T2: brute force must try every password — no collisions
        let key1 = derive_master_key(b"password-one", TEST_SALT, &fast_params()).unwrap();
        let key2 = derive_master_key(b"password-two", TEST_SALT, &fast_params()).unwrap();
        assert_ne!(key1.expose_secret(), key2.expose_secret());
    }

    #[test]
    fn different_salts_produce_different_keys() {
        // T2: each vault has a unique salt — same password, different vault = different key
        let salt2 = b"omoide_test_salt_different_vault";
        let key1 = derive_master_key(TEST_PASSWORD, TEST_SALT, &fast_params()).unwrap();
        let key2 = derive_master_key(TEST_PASSWORD, salt2, &fast_params()).unwrap();
        assert_ne!(key1.expose_secret(), key2.expose_secret());
    }

    #[test]
    fn entry_enc_and_seed_wrap_tags_produce_different_keys() {
        // T3: critical — seed-wrap key must never equal any entry-enc key
        let master = derive_master_key(TEST_PASSWORD, TEST_SALT, &fast_params()).unwrap();
        let entry_id = [0u8; 16]; // same entry_id deliberately

        let entry_key = derive_entry_key(&master, &entry_id, b"entry-enc").unwrap();
        let seed_key  = derive_entry_key(&master, &entry_id, b"seed-wrap").unwrap();

        assert_ne!(entry_key.expose_secret(), seed_key.expose_secret());
    }

    #[test]
    fn different_entry_ids_produce_different_keys() {
        // two vault entries must have independent keys
        let master = derive_master_key(TEST_PASSWORD, TEST_SALT, &fast_params()).unwrap();
        let id1 = [1u8; 16];
        let id2 = [2u8; 16];

        let key1 = derive_entry_key(&master, &id1, b"entry-enc").unwrap();
        let key2 = derive_entry_key(&master, &id2, b"entry-enc").unwrap();

        assert_ne!(key1.expose_secret(), key2.expose_secret());
    }
}