use aes_gcm::{
    aead::{AeadInPlace, KeyInit},
    Aes256Gcm, Nonce,
};
use omoide_env::{AES_NONCE_SIZE, HEADER_AAD_SIZE};

use crate::{CryptoError, EntryKey};

pub fn encrypt_entry(
    key: &EntryKey,
    nonce_bytes: &[u8; AES_NONCE_SIZE],
    aad: &[u8; HEADER_AAD_SIZE],
    plaintext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(key.expose_secret())
        .map_err(|_| CryptoError::InvalidKeyLength)?;
    let nonce = Nonce::from_slice(nonce_bytes);

    // Create a buffer for the ciphertext, returning a vector matching the length
    // of plaintext + 16 bytes for the authentication tag as specified by AeadInPlace
    let mut buffer = plaintext.to_vec();
    cipher
        .encrypt_in_place(nonce, aad, &mut buffer)
        .map_err(|_| CryptoError::EncryptionFailed)?;

    Ok(buffer)
}

pub fn decrypt_entry(
    key: &EntryKey,
    nonce_bytes: &[u8; AES_NONCE_SIZE],
    aad: &[u8; HEADER_AAD_SIZE],
    ciphertext: &[u8],
) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(key.expose_secret())
        .map_err(|_| CryptoError::InvalidKeyLength)?;
    let nonce = Nonce::from_slice(nonce_bytes);

    let mut buffer = ciphertext.to_vec();
    cipher
        .decrypt_in_place(nonce, aad, &mut buffer)
        .map_err(|_| CryptoError::DecryptionFailed)?;

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_key() -> EntryKey {
        let mut key = EntryKey::new_zeroed();
        for (i, b) in key.expose_mut().iter_mut().enumerate() {
            *b = i as u8;
        }
        key
    }

    #[test]
    fn correct_key_decrypts_entry() {
        let key = make_test_key();
        let nonce = [1u8; AES_NONCE_SIZE];
        let aad = [2u8; HEADER_AAD_SIZE];
        let plaintext = b"hello world";

        let ciphertext = encrypt_entry(&key, &nonce, &aad, plaintext).unwrap();
        let decrypted = decrypt_entry(&key, &nonce, &aad, &ciphertext).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn wrong_key_returns_decryption_failed() {
        let key1 = make_test_key();
        let mut key2 = make_test_key();
        key2.expose_mut()[0] ^= 1; // alter the key

        let nonce = [1u8; AES_NONCE_SIZE];
        let aad = [2u8; HEADER_AAD_SIZE];
        let plaintext = b"hello world";

        let ciphertext = encrypt_entry(&key1, &nonce, &aad, plaintext).unwrap();

        // Decrypt with wrong key
        let err = decrypt_entry(&key2, &nonce, &aad, &ciphertext).unwrap_err();
        assert!(matches!(err, CryptoError::DecryptionFailed));
    }

    #[test]
    fn flipped_ciphertext_byte_fails() {
        let key = make_test_key();
        let nonce = [1u8; AES_NONCE_SIZE];
        let aad = [2u8; HEADER_AAD_SIZE];
        let plaintext = b"hello world";

        let mut ciphertext = encrypt_entry(&key, &nonce, &aad, plaintext).unwrap();
        ciphertext[0] ^= 1; // bitwise XOR to alter a byte of ciphertext

        let err = decrypt_entry(&key, &nonce, &aad, &ciphertext).unwrap_err();
        assert!(matches!(err, CryptoError::DecryptionFailed));
    }

    #[test]
    fn entry_from_different_vault_fails_aad_check() {
        let key = make_test_key();
        let nonce = [1u8; AES_NONCE_SIZE];
        let aad1 = [2u8; HEADER_AAD_SIZE];
        let aad2 = [3u8; HEADER_AAD_SIZE]; // different vault AAD
        let plaintext = b"hello world";

        let ciphertext = encrypt_entry(&key, &nonce, &aad1, plaintext).unwrap();

        let err = decrypt_entry(&key, &nonce, &aad2, &ciphertext).unwrap_err();
        assert!(matches!(err, CryptoError::DecryptionFailed));
    }
}
