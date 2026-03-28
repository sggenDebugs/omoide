
use zeroize::{Zeroize, ZeroizeOnDrop};

/// 32-byte per-entry encryption key derived via HKDF from MasterKey.
///
/// Shorter-lived than MasterKey — created, used for one encrypt/decrypt
/// operation, then dropped immediately. Never stored anywhere.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct EntryKey([u8; 32]);

impl EntryKey {
    pub fn new_zeroed() -> Self {
        Self([0u8; 32])
    }

    pub fn expose_secret(&self) -> &[u8; 32] {
        &self.0
    }

    pub(crate) fn expose_mut(&mut self) -> &mut [u8; 32] {
        &mut self.0
    }
}

impl std::fmt::Debug for EntryKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EntryKey([REDACTED])")
    }
}