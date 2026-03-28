
use crate::error::CryptoError;
use crate::types::{MasterKey, EntryKey};
use argon2::{Argon2, Algorithm, Version, Params};
use hkdf::Hkdf;
use sha2::Sha256;
/// KDF parameters frozen at vault creation time.
/// Stored in vault header plaintext so the vault can be reopened
/// without knowing params in advance.
/// DO NOT change defaults without running `omoide bench-kdf` and updating ADR-001.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KdfParams {
    pub memory_cost: u32, // memory in KiB — default: 19456 (19 MiB)
    pub time_cost: u32, // iterations   — default: 2
    pub parallelism_cost: u32, // parallelism  — default: 1
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            memory_cost: 19456,
            time_cost: 2,
            parallelism_cost: 1,
        }
    }
}

pub fn derive_master_key(
    password: &[u8],
    salt: &[u8; 32],
    params: &KdfParams,
) -> Result<MasterKey, CryptoError> {
    let argon2 = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(params.memory_cost, params.time_cost, params.parallelism_cost, Some(32))?,
    );
    let mut key = MasterKey::new_zeroed();
    argon2.hash_password_into(password, salt, key.expose_mut())?;
    Ok(key)
}

pub fn derive_entry_key(
    master: &MasterKey,
    entry_id: &[u8; 16],
    info: &[u8],
) -> Result<EntryKey, CryptoError> {
    let hashed_key = Hkdf::<Sha256>::new(Some(entry_id), master.expose_secret());
    let mut key = EntryKey::new_zeroed();
    hashed_key.expand(info, key.expose_mut())
        .map_err(|_| CryptoError::HkdfExpand)?;
    Ok(key)
}
