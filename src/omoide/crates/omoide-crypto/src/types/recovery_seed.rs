use zeroize::ZeroizeOnDrop;

/// 64-byte BIP39 seed derived from mnemonic phrase.
///
/// # Security contract (T3)
/// - MUST be zeroized within 60 seconds of creation
/// - MUST be zeroized immediately if the application window loses focus
/// - MUST NOT be written to disk, logs, clipboard, or error messages
/// - Caller is responsible for enforcing the 60s timeout
///
/// Dropping this value satisfies the zeroize requirement automatically,
/// but the timeout must be enforced by the UI layer.
#[derive(ZeroizeOnDrop)]
pub struct RecoverySeed([u8; 64]);

impl RecoverySeed {
    pub fn new_zeroed() -> Self {
        Self([0u8; 64])
    }

    pub fn expose_secret(&self) -> &[u8; 64] {
        &self.0
    }

    pub(crate) fn expose_mut(&mut self) -> &mut [u8; 64] { // to be used
        &mut self.0
    }
}

impl std::fmt::Debug for RecoverySeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RecoverySeed([REDACTED])")
    }
}