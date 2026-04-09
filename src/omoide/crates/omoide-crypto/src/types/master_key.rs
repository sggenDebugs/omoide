use zeroize::{Zeroize, ZeroizeOnDrop};

// Constant Definitions
const MASTER_KEY_SIZE: usize = 32;

/// 32-byte master key derived from master password via Argon2id.
///
/// **Security guarantees**
/// - Zeroed on Drop via ZeroizeOnDrop
/// - Only accessible via expose_secret() — never implements Display or Debug
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct MasterKey([u8; MASTER_KEY_SIZE]);

impl MasterKey {
    /// Allocates a zeroed buffer. Call this before populating
    /// with KDF output.
    ///
    /// Returns itself with values of zeroes.
    pub fn new_zeroed() -> Self {
        Self([0u8; MASTER_KEY_SIZE])
    }

    /// Read-only access. Every call site is visible during audit.
    pub fn expose_secret(&self) -> &[u8; MASTER_KEY_SIZE] {
        &self.0
    }

    /// Give write access to KDF derivation to populate the buffer.
    pub(crate) fn expose_mut(&mut self) -> &mut [u8; MASTER_KEY_SIZE] {
        &mut self.0
    }
}

// Explicitly deny these — a MasterKey must never be printed or cloned
impl std::fmt::Debug for MasterKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("MasterKey([REDACTED])")
    }
}
