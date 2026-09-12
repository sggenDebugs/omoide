#![no_main]

use libfuzzer_sys::fuzz_target;
use omoide_crypto::{decrypt_entry, EntryKey};
use omoide_env::{AES_NONCE_SIZE, HEADER_AAD_SIZE};

// Key bytes length: 32 bytes for AES-256 (Argon2id output size).
const KEY_SIZE: usize = 32;

/// Minimum input: nonce + AAD + key material, with ciphertext being zero or more bytes.
const MIN_LEN: usize = AES_NONCE_SIZE + HEADER_AAD_SIZE + KEY_SIZE;

fuzz_target!(|data: &[u8]| {
    if data.len() < MIN_LEN {
        return;
    }

    let nonce: &[u8; AES_NONCE_SIZE] = data[..AES_NONCE_SIZE].try_into().unwrap();
    let aad: &[u8; HEADER_AAD_SIZE] = data[AES_NONCE_SIZE..AES_NONCE_SIZE + HEADER_AAD_SIZE]
        .try_into()
        .unwrap();
    let key_bytes: [u8; KEY_SIZE] = data
        [AES_NONCE_SIZE + HEADER_AAD_SIZE..AES_NONCE_SIZE + HEADER_AAD_SIZE + KEY_SIZE]
        .try_into()
        .unwrap();
    let ciphertext = &data[AES_NONCE_SIZE + HEADER_AAD_SIZE + KEY_SIZE..];

    let key = EntryKey::from_bytes(&key_bytes);
    // Must not panic — only Ok or Err::DecryptionFailed are valid outcomes.
    let _ = decrypt_entry(&key, nonce, aad, ciphertext);
});
