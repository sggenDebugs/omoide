// crates/omoide-core/src/lib.rs

pub mod error;
pub mod security;
pub mod vault;
pub mod srs;

pub use error::VaultError;
pub use security::{mlock_secret, suppress_core_dumps};
pub use vault::{open, seal};
