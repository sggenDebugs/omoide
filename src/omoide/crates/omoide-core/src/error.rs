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

/// Errors produced by the auth orchestrator state machine.
#[derive(Debug, Error)]
pub enum AuthError {
    #[error("vault is locked")]
    VaultLocked,

    #[error("vault is already unlocked")]
    AlreadyUnlocked,

    #[error("not currently awaiting a reprompt")]
    NotAwaitingReprompt,

    /// Returned when a reprompt attempt fails but retries remain.
    /// The caller may call submit_reprompt() again while retries_left > 0.
    #[error("reprompt failed: incorrect password ({retries_left} retries left)")]
    RepromptFailed { retries_left: u8 },

    /// Returned when all retry attempts have been consumed.
    /// The vault has been re-sealed with the penalty SRS state and the
    /// session has transitioned to Locked. All secrets have been zeroized.
    #[error("All reprompt retries exhausted. Vault locked and secrets zeroized.")]
    AllRetriesExhausted,

    /// Returned (and the vault locked) when the reprompt window timed out.
    #[error("Reprompt timed out. Vault locked and secrets zeroized.")]
    RepromptTimedOut,

    #[error("Vault error: {0}")]
    Vault(#[from] VaultError),

    #[error("Crypto error: {0}")]
    Crypto(#[from] omoide_crypto::CryptoError),
}
