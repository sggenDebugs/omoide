// crates/omoide-core/src/lib.rs

pub mod security;

pub use security::{suppress_core_dumps, mlock_secret};