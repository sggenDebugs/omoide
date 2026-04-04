pub mod error;
pub mod schema;

pub use error::FormatError;
pub use schema::{
    Entry, EncryptedEntry, VaultFile, VaultHeader,
};