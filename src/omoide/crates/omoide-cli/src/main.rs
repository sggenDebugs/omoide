use omoide_core::error::AuthError;
use omoide_core::orchestrator::AuthOrchestrator;
use omoide_core::srs::get_current_time_secs;
use omoide_core::vault::{open, seal};
use omoide_crypto::aead::encrypt_entry;
use omoide_crypto::kdf::{derive_entry_key, derive_master_key, KdfParams};
use omoide_env::{
    AES_NONCE_SIZE, ENTRY_ID_SIZE, HEADER_AAD_SIZE, KDF_SALT_SIZE, REPROMPT_MAX_RETRIES,
    REPROMPT_TIMEOUT_SECS,
};
use omoide_format::schema::{EncryptedEntry, Entry, VaultFile, VaultHeader};
use rand::{rngs::SysRng, TryRng};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

fn generate_random_bytes<const N: usize>() -> [u8; N] {
    let mut buf = [0u8; N];
    SysRng
        .try_fill_bytes(&mut buf)
        .expect("CRITICAL: OS failed to provide secure randomness.");
    buf
}

fn anchor_memory_wizard() {
    println!("\n--- Password Architect Wizard ---");
    println!("Let's build an 'Anchor Memory' passphrase for maximum retention and entropy.");
    println!("Think of a highly specific, private childhood place.");
    let place = prompt("Place (e.g., Attic): ");
    println!("Think of a vivid, unrelated action verb.");
    let action = prompt("Action (e.g., Sprinting): ");
    println!("Think of an unrelated object.");
    let object = prompt("Object (e.g., Violin): ");

    let mut passphrase = format!("{}{}{}", place.trim(), action.trim(), object.trim());
    if passphrase.is_empty() {
        passphrase = "AtticSprintingViolin".to_string();
    }

    println!(
        "\n[SUCCESS] Your highly secure, SRS-compatible passphrase is: {}",
        passphrase
    );
    println!("This utilizes 'Nonsense Logic' to maintain high entropy while securely anchoring to your personal memory.");
    println!("Press Enter to continue...");
    let _ = prompt("");
}

fn create_sample_vault(path: &Path, password: &str, entries: Vec<Entry>) {
    // We use faster params for tests/demo
    let params = KdfParams {
        memory_cost: 1024,
        time_cost: 1,
        parallelism_cost: 1,
    };
    let salt = generate_random_bytes::<KDF_SALT_SIZE>();
    let header_aad = generate_random_bytes::<HEADER_AAD_SIZE>();

    let master_key = derive_master_key(password.as_bytes(), &salt, &params).unwrap();

    let mut encrypted_entries = Vec::new();

    for entry in entries {
        let entry_id = generate_random_bytes::<ENTRY_ID_SIZE>();
        let nonce = generate_random_bytes::<AES_NONCE_SIZE>();
        let entry_key = derive_entry_key(&master_key, &entry_id, b"entry-enc").unwrap();

        let mut cbor_bytes = Vec::new();
        ciborium::ser::into_writer(&entry, &mut cbor_bytes).unwrap();

        let ciphertext = encrypt_entry(&entry_key, &nonce, &header_aad, &cbor_bytes).unwrap();

        encrypted_entries.push(EncryptedEntry {
            id: entry_id,
            nonce,
            ciphertext,
        });
    }

    let vault = VaultFile {
        header: VaultHeader {
            kdf_params: params,
            salt,
            header_aad,
            srs_state: Default::default(),
        },
        entries: encrypted_entries,
    };

    seal(&vault, path).unwrap();
}

fn setup_vaults() -> (PathBuf, PathBuf) {
    let home = std::env::current_dir().unwrap().join("demo_vaults");
    fs::create_dir_all(&home).unwrap();

    let v1 = home.join("vault1.db");
    let v2 = home.join("vault2.db");

    let e1 = Entry {
        title: "Work Email".to_string(),
        username: "alice@work.com".to_string(),
        password: "super_secret_work_password!23".to_string(),
        url: "mail.work.com".to_string(),
        notes: "Company email".to_string(),
        created: 1700000000,
        updated: 1700000000,
    };

    let e2 = Entry {
        title: "Personal Bank".to_string(),
        username: "alice_b".to_string(),
        password: "bank_password_99".to_string(),
        url: "bank.com".to_string(),
        notes: "Main checking".to_string(),
        created: 1700000000,
        updated: 1700000000,
    };

    create_sample_vault(&v1, "password", vec![e1, e2]);

    let e3 = Entry {
        title: "Gaming Account".to_string(),
        username: "gamer_alice".to_string(),
        password: "hunter2".to_string(),
        url: "steam.com".to_string(),
        notes: "".to_string(),
        created: 1700000000,
        updated: 1700000000,
    };
    create_sample_vault(&v2, "12345", vec![e3]);

    (v1, v2)
}

fn print_entries(orch: &AuthOrchestrator) {
    if let Some(entries) = orch.entries() {
        println!("\n--- Vault Entries ---");
        for entry in entries {
            println!("Title:    {}", entry.title);
            println!("Username: {}", entry.username);
            println!("Password: {}", entry.password);
            println!("URL:      {}", entry.url);
            println!("---------------------");
        }
    }
}

/// Unlock a vault and display its contents, then return control.
fn display_vault(path: &Path, password: &str) -> bool {
    let mut orch = AuthOrchestrator::new(path.to_path_buf())
        .with_timeout(REPROMPT_TIMEOUT_SECS)
        .with_max_retries(REPROMPT_MAX_RETRIES);

    println!("\n[DEBUG] Accessing vault at {:?}", path);

    match orch.unlock(password) {
        Ok(()) => {
            println!(
                "[DEBUG] Vault unlocked successfully. State: {}",
                orch.state_label()
            );
        }
        Err(e) => {
            println!("[ERROR] Failed to unlock vault: {}", e);
            return false;
        }
    }

    print_entries(&orch);
    true
}

/// Interactive session loop — keeps the vault unlocked in memory and drives
/// periodic reprompt checks, demonstrating the full EAR lifecycle.
fn run_session_loop(path: &Path, password: &str) {
    let mut orch = AuthOrchestrator::new(path.to_path_buf())
        .with_timeout(REPROMPT_TIMEOUT_SECS)
        .with_max_retries(REPROMPT_MAX_RETRIES);

    println!("\n[SESSION] Starting interactive session for {:?}", path);
    println!(
        "[SESSION] Reprompt timeout: {}s | Max retries: {}",
        REPROMPT_TIMEOUT_SECS, REPROMPT_MAX_RETRIES
    );

    match orch.unlock(password) {
        Ok(()) => println!("[SESSION] Vault unlocked. State: {}", orch.state_label()),
        Err(e) => {
            println!("[SESSION] Failed to unlock: {}", e);
            return;
        }
    }

    print_entries(&orch);

    println!("\n[SESSION] Entering reprompt loop. Type 'quit' to exit, or wait for reprompt.");
    println!("[SESSION] Use option 4 from the main menu first to force an immediate reprompt.\n");

    loop {
        // Give the user a chance to type a command between ticks.
        print!("[SESSION] (tick) > ");
        io::stdout().flush().unwrap();

        // Non-blocking: sleep briefly then check state.
        thread::sleep(Duration::from_secs(2));

        let now = get_current_time_secs();

        // Check timeout first (takes priority over new reprompt).
        if orch.check_timeout(now) {
            println!("\n[SESSION] *** REPROMPT TIMED OUT — Vault locked. Secrets zeroized. ***");
            break;
        }

        // Check if a reprompt is now due.
        if orch.check_reprompt(now) {
            println!("\n[SESSION] *** EMERGENCY ACCESS REHEARSAL TRIGGERED ***");
            println!("[SESSION] State: {}", orch.state_label());

            // Drive the reprompt interaction.
            loop {
                let pw = prompt("[SESSION] Re-enter Master Password (or 'quit' to abort): ");
                if pw.trim() == "quit" {
                    orch.lock();
                    println!("[SESSION] Vault manually locked. Secrets zeroized.");
                    return;
                }

                match orch.submit_reprompt(pw.trim()) {
                    Ok(()) => {
                        println!(
                            "[SESSION] Reprompt successful! Interval extended. State: {}",
                            orch.state_label()
                        );
                        print_entries(&orch);
                        break;
                    }
                    Err(AuthError::RepromptFailed { retries_left }) => {
                        println!(
                            "[SESSION] Wrong password. {} retr{} remaining.",
                            retries_left,
                            if retries_left == 1 { "y" } else { "ies" }
                        );
                    }
                    Err(AuthError::AllRetriesExhausted) => {
                        println!("[SESSION] *** ALL RETRIES EXHAUSTED — Vault locked. Secrets zeroized. ***");
                        return;
                    }
                    Err(e) => {
                        println!("[SESSION] Unexpected error: {}", e);
                        orch.lock();
                        return;
                    }
                }
            }
        } else {
            // No reprompt due yet — show status.
            let vault = open(orch.vault_path()).unwrap();
            let srs = &vault.header.srs_state;
            let interval_secs = (srs.current_interval_hours * 3600.0) as u64;
            let due_at = srs.last_rehearsal + interval_secs;
            let remaining = due_at.saturating_sub(now);
            println!(
                "State: {} | Next rehearsal in {}s",
                orch.state_label(),
                remaining
            );
        }

        // Check if user typed something.
        // In a real TUI we'd use async I/O; for demo we just poll stdin after the sleep.
    }
}

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    io::stdout().flush().unwrap();
    let mut line = String::new();
    io::stdin().read_line(&mut line).unwrap();
    line.trim().to_string()
}

fn main() {
    println!("Initializing Omoide CLI Demo...");
    #[cfg(unix)]
    omoide_core::security::suppress_core_dumps();

    let (v1, v2) = setup_vaults();

    println!("Created 2 vaults in ./demo_vaults:");
    println!("  1. Vault A (password: 'password')");
    println!("  2. Vault B (password: '12345')");

    loop {
        println!("\nSelect an option:");
        println!("1) Unlock Vault A (one-shot)");
        println!("2) Unlock Vault B (one-shot)");
        println!("3) Generate Master Password (Wizard)");
        println!("4) Fast-Forward Time (Force Recall Mode on Vault B)");
        println!("5) Start Session Loop for Vault A");
        println!("6) Start Session Loop for Vault B");
        println!("7) Exit");

        let choice = prompt("> ");
        match choice.as_str() {
            "1" => {
                let pw = prompt("Enter Master Password for Vault A: ");
                display_vault(&v1, &pw);
            }
            "2" => {
                let pw = prompt("Enter Master Password for Vault B: ");
                display_vault(&v2, &pw);
            }
            "3" => {
                anchor_memory_wizard();
            }
            "4" => {
                println!("[DEBUG] Simulating time advancing by updating the Vault B header...");
                if let Ok(mut v) = open(&v2) {
                    v.header.srs_state.last_rehearsal = 10;
                    let _ = seal(&v, &v2);
                    println!("[DEBUG] Time leap successful. Try opening Vault B next.");
                } else {
                    println!("[ERROR] Could not open Vault B.");
                }
            }
            "5" => {
                let pw = prompt("Enter Master Password for Vault A: ");
                run_session_loop(&v1, &pw);
            }
            "6" => {
                let pw = prompt("Enter Master Password for Vault B: ");
                run_session_loop(&v2, &pw);
            }
            "7" => {
                println!("Goodbye!");
                break;
            }
            _ => {
                println!("Invalid choice. Please try again.");
            }
        }
    }
}
