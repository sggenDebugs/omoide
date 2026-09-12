use zeroize::{Zeroize, ZeroizeOnDrop};

// Constant Definitions
const ENTRY_KEY_SIZE: usize = 32;

/// 32-byte per-entry encryption key derived via HKDF from MasterKey.
///
/// Shorter-lived than MasterKey — created, used for one encrypt/decrypt
/// operation, then dropped immediately. Never stored anywhere.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct EntryKey([u8; ENTRY_KEY_SIZE]);

impl EntryKey {
    pub fn new_zeroed() -> Self {
        Self([0u8; ENTRY_KEY_SIZE])
    }

    /// Construct an `EntryKey` from a raw 32-byte array.
    /// Intended for use in fuzz testing and test helpers where key material
    /// is already available as a byte array.
    pub fn from_bytes(bytes: &[u8; ENTRY_KEY_SIZE]) -> Self {
        Self(*bytes)
    }

    pub fn expose_secret(&self) -> &[u8; ENTRY_KEY_SIZE] {
        &self.0
    }

    pub(crate) fn expose_mut(&mut self) -> &mut [u8; ENTRY_KEY_SIZE] {
        &mut self.0
    }
}

impl std::fmt::Debug for EntryKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("EntryKey([REDACTED])")
    }
}
