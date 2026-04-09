use ciborium::{de::Error as CborDeError, ser::Error as CborSerError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VaultError {
    #[error("invalid magic bytes — not an omoide vault file")]
    InvalidMagic,

    #[error("CBOR serialization error: {0}")]
    CborSerialize(#[from] CborSerError<std::io::Error>),

    #[error("CBOR deserialization error: {0}")]
    CborDeserialize(#[from] CborDeError<std::io::Error>),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
