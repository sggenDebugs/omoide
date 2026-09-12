use omoide_crypto::{
    decrypt_entry, derive_entry_key, derive_master_key, encrypt_entry, CryptoError, KdfParams,
};
use omoide_env::{AES_NONCE_SIZE, ENTRY_ID_SIZE, HEADER_AAD_SIZE, KDF_SALT_SIZE};
use zeroize::Zeroize;

// Helper constants for tests
const TEST_PASSWORD: &[u8] = b"correct-horse-battery-staple";
const WRONG_PASSWORD: &[u8] = b"wrong-horse-battery-staple";
const DUMMY_SALT: [u8; KDF_SALT_SIZE] = [1u8; KDF_SALT_SIZE];
const ENTRY_ID: [u8; ENTRY_ID_SIZE] = [2u8; ENTRY_ID_SIZE];
const NONCE: [u8; AES_NONCE_SIZE] = [3u8; AES_NONCE_SIZE];
const AAD: [u8; HEADER_AAD_SIZE] = [4u8; HEADER_AAD_SIZE];

// Use minimal params for integration tests so they don't take 500ms each
fn fast_params() -> KdfParams {
    KdfParams {
        memory_cost: 1024,
        time_cost: 1,
        parallelism_cost: 1,
    }
}

#[test]
fn correct_password_decrypts_entry() {
    let master = derive_master_key(TEST_PASSWORD, &DUMMY_SALT, &fast_params()).unwrap();
    let entry_key = derive_entry_key(&master, &ENTRY_ID, b"entry-enc").unwrap();

    let plaintext = b"secret vault entry data";

    let ciphertext = encrypt_entry(&entry_key, &NONCE, &AAD, plaintext).unwrap();
    let decrypted = decrypt_entry(&entry_key, &NONCE, &AAD, &ciphertext).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn wrong_password_returns_decryption_failed() {
    let params = fast_params();
    let master1 = derive_master_key(TEST_PASSWORD, &DUMMY_SALT, &params).unwrap();
    let entry_key1 = derive_entry_key(&master1, &ENTRY_ID, b"entry-enc").unwrap();

    let master2 = derive_master_key(WRONG_PASSWORD, &DUMMY_SALT, &params).unwrap();
    let entry_key2 = derive_entry_key(&master2, &ENTRY_ID, b"entry-enc").unwrap();

    let plaintext = b"secret vault entry data";
    let ciphertext = encrypt_entry(&entry_key1, &NONCE, &AAD, plaintext).unwrap();

    let err = decrypt_entry(&entry_key2, &NONCE, &AAD, &ciphertext).unwrap_err();
    assert!(matches!(err, CryptoError::DecryptionFailed));
}

#[test]
fn flipped_ciphertext_byte_fails() {
    let master = derive_master_key(TEST_PASSWORD, &DUMMY_SALT, &fast_params()).unwrap();
    let entry_key = derive_entry_key(&master, &ENTRY_ID, b"entry-enc").unwrap();

    let plaintext = b"secret vault entry data";
    let mut ciphertext = encrypt_entry(&entry_key, &NONCE, &AAD, plaintext).unwrap();

    // tamper the ciphertext
    ciphertext[5] ^= 1;

    let err = decrypt_entry(&entry_key, &NONCE, &AAD, &ciphertext).unwrap_err();
    assert!(matches!(err, CryptoError::DecryptionFailed));
}

#[test]
fn entry_from_different_vault_fails_aad_check() {
    let master = derive_master_key(TEST_PASSWORD, &DUMMY_SALT, &fast_params()).unwrap();
    let entry_key = derive_entry_key(&master, &ENTRY_ID, b"entry-enc").unwrap();

    let aad_1 = [4u8; HEADER_AAD_SIZE]; // Vault 1
    let aad_2 = [5u8; HEADER_AAD_SIZE]; // Vault 2 (different vault header)

    let plaintext = b"secret vault entry data";
    let ciphertext = encrypt_entry(&entry_key, &NONCE, &aad_1, plaintext).unwrap();

    // Attacker invades entry to Vault 2 and tries to decrypt with Vault 2's AAD context
    let err = decrypt_entry(&entry_key, &NONCE, &aad_2, &ciphertext).unwrap_err();
    assert!(matches!(err, CryptoError::DecryptionFailed));
}

#[test]
fn master_key_zeroization() {
    let mut master = derive_master_key(TEST_PASSWORD, &DUMMY_SALT, &fast_params()).unwrap();
    master.zeroize();

    // Explicitly check that `zeroize` wiped the buffer inside the MasterKey wrapper.
    // By definition, this guarantees that when `.drop()` is implicitly called,
    // it will zeroize automatically because it implements `ZeroizeOnDrop` which invokes `.zeroize()`.
    assert_eq!(master.expose_secret(), &[0u8; 32]);
}

#[test]
fn seed_wrap_key_differs_from_entry_enc_key() {
    let master = derive_master_key(TEST_PASSWORD, &DUMMY_SALT, &fast_params()).unwrap();

    let k1 = derive_entry_key(&master, &ENTRY_ID, b"entry-enc").unwrap();
    let k2 = derive_entry_key(&master, &ENTRY_ID, b"seed-wrap").unwrap();

    assert_ne!(k1.expose_secret(), k2.expose_secret());
}
