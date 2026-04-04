use thiserror::Error;
use ciborium::{ser::Error as CborSerError, de::Error as CborDeError};

#[derive(Debug, Error)]
pub enum FormatError {
    #[error("invalid magic bytes — not an omoide vault file")]
    InvalidMagic,

    #[error("invalid version number: {0}")]
    InvalidVersion(u16),

    #[error("CBOR serialization error")]
    CborSerialize(#[from] CborSerError<std::io::Error>),

    #[error("CBOR deserialization error")]
    CborDeserialize(#[from] CborDeError<std::io::Error>),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error)
}