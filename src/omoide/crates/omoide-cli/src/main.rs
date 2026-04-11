use omoide_core::vault::{open, seal};
use omoide_crypto::aead::{decrypt_entry, encrypt_entry};
use omoide_crypto::kdf::{derive_entry_key, derive_master_key, KdfParams};
use omoide_env::{AES_NONCE_SIZE, ENTRY_ID_SIZE, HEADER_AAD_SIZE, KDF_SALT_SIZE};
use omoide_format::schema::{EncryptedEntry, Entry, VaultFile, VaultHeader};
use rand::{rngs::OsRng, RngCore};
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

fn generate_random_bytes<const N: usize>() -> [u8; N] {
    let mut buf = [0u8; N];
    OsRng.fill_bytes(&mut buf);
    buf
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

fn display_vault(path: &Path, password: &str) -> bool {
    println!("\n[DEBUG] Accessing locked vault at {:?}", path);
    let vault = match open(path) {
        Ok(v) => {
            println!("[DEBUG] Vault file loaded successfully into memory.");
            println!(
                "[DEBUG] Vault header contains: KDF salt ({} bytes), AAD ({} bytes)",
                v.header.salt.len(),
                v.header.header_aad.len()
            );
            println!(
                "[DEBUG] Locked vault contains {} encrypted entries.",
                v.entries.len()
            );
            v
        }
        Err(e) => {
            println!(" Failed to read vault file: {}", e);
            return false;
        }
    };

    println!("[DEBUG] Applying KDF (Argon2id) to derive master key from password...");
    let master_key = match derive_master_key(
        password.as_bytes(),
        &vault.header.salt,
        &vault.header.kdf_params,
    ) {
        Ok(key) => {
            println!("[DEBUG] KDF complete. Master key derived successfully.");
            key
        }
        Err(e) => {
            println!(" KDF failed: {}", e);
            return false;
        }
    };

    let mut decrypted_entries = Vec::new();
    println!("[DEBUG] Processing encrypted entries...");
    for (i, enc_entry) in vault.entries.into_iter().enumerate() {
        println!("[DEBUG] -> Entry {}: Deriving unique entry key...", i + 1);
        let entry_key = match derive_entry_key(&master_key, &enc_entry.id, b"entry-enc") {
            Ok(k) => k,
            Err(e) => {
                println!("[DEBUG] -> Entry {}: Key derivation failed: {}", i + 1, e);
                continue;
            }
        };

        println!(
            "[DEBUG] -> Entry {}: Decrypting ciphertext ({} bytes)...",
            i + 1,
            enc_entry.ciphertext.len()
        );
        match decrypt_entry(
            &entry_key,
            &enc_entry.nonce,
            &vault.header.header_aad,
            &enc_entry.ciphertext,
        ) {
            Ok(pt) => {
                println!(
                    "[DEBUG] -> Entry {}: Decrypted successfully. Unlocked! decoding CBOR...",
                    i + 1
                );
                let entry: Entry = ciborium::de::from_reader(pt.as_slice()).unwrap();
                decrypted_entries.push(entry);
            }
            Err(e) => {
                println!("[DEBUG] Authentication failed for entry {}! Incorrect master password or corrupted entry: {}", i + 1, e);
                return false;
            }
        }
    }

    println!("\n Master Password Correct! Vault completely unlocked.");
    println!("--- Vault Entries ---");
    for e in decrypted_entries {
        println!("Title: {}", e.title);
        println!("Username: {}", e.username);
        println!("Password: {}", e.password);
        println!("URL: {}", e.url);
        println!("---------------------");
    }
    true
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
        println!("\nSelect a vault to unlock:");
        println!("1) Vault A");
        println!("2) Vault B");
        println!("3) Exit");

        let choice = prompt("> ");
        let selected_path = match choice.as_str() {
            "1" => &v1,
            "2" => &v2,
            "3" => {
                println!("Goodbye!");
                break;
            }
            _ => {
                println!("Invalid choice. Expected 1, 2, or 3.");
                continue;
            }
        };

        let pw = prompt("Enter Master Password: ");
        display_vault(selected_path, &pw);
    }
}
