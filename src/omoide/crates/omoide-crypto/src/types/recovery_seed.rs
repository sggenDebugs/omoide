use zeroize::ZeroizeOnDrop;

// Constant Definitions
const RECV_SEED_SIZE: usize = 64;

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
pub struct RecoverySeed([u8; RECV_SEED_SIZE]);

impl RecoverySeed {
    pub fn new_zeroed() -> Self {
        Self([0u8; RECV_SEED_SIZE])
    }

    pub fn expose_secret(&self) -> &[u8; RECV_SEED_SIZE] {
        &self.0
    }

    /// Allow dead code first, then remove this method and its uses once Seed backup is implemented.
    #[allow(dead_code)]
    pub(crate) fn expose_mut(&mut self) -> &mut [u8; RECV_SEED_SIZE] { // to be used
        &mut self.0
    }
}

impl std::fmt::Debug for RecoverySeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RecoverySeed([REDACTED])")
    }
}