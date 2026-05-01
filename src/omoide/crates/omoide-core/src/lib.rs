// crates/omoide-core/src/lib.rs

pub mod error;
pub mod orchestrator;
pub mod security;
pub mod srs;
pub mod vault;

pub use error::{AuthError, VaultError};
pub use orchestrator::AuthOrchestrator;
pub use security::{mlock_secret, suppress_core_dumps};
pub use vault::{open, seal};
