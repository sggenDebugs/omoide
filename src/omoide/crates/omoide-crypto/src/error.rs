use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("key derivation failed")]
    Kdf(#[from] argon2::Error),

    #[error("HKDF expand failed")]
    HkdfExpand,

    #[error("decryption failed")]
    DecryptionFailed,

    #[error("encryption failed")]
    EncryptionFailed,

    #[error("invalid key length")]
    InvalidKeyLength,
}
