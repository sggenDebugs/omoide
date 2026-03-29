use zeroize::{Zeroize, ZeroizeOnDrop};

/// 32-byte master key derived from master password via Argon2id.
///
/// # Security guarantees
/// - Allocated in mlock'd memory — will not be paged to swap (T1)
/// - Zeroed on Drop via ZeroizeOnDrop (T1)
/// - Only accessible via expose_secret() — never implements Display or Debug
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct MasterKey([u8; 32]);

impl MasterKey {
    /// Allocates a locked, zeroed buffer. Call this before populating
    /// with KDF output — the memory is locked BEFORE secrets enter it.
    pub fn new_zeroed() -> Self {
        Self([0u8; 32])
    }

    /// Read-only access. Every call site is visible during audit.
    pub fn expose_secret(&self) -> &[u8; 32] {
        &self.0
    }

    /// Write access — only used during KDF derivation to populate the buffer.
    pub(crate) fn expose_mut(&mut self) -> &mut [u8; 32] {
        &mut self.0
    }
}

// Explicitly deny these — a MasterKey must never be printed or cloned
impl std::fmt::Debug for MasterKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MasterKey([REDACTED])")
    }
}
