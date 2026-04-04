#![no_std] // optional, if you want minimal runtime

/// Size of an entry UUID in bytes — used as HKDF salt and in EncryptedEntry.id.
pub const ENTRY_ID_SIZE: usize = 16;

/// Size of KDF salt in bytes.
pub const KDF_SALT_SIZE: usize = 32;

/// Memory cost of KDF in KiB.
pub const KDFPARAMS_MEM_COST: u32 = 19456;

/// Time cost of KDF.
pub const KDFPARAMS_TIME_COST: u32 = 2;

/// Parallelism cost of KDF.
pub const KDFPARAMS_PARALLEL_COST: u32 = 1;

/// Size of header AAD in bytes.
pub const HEADER_AAD_SIZE: usize = 32;

/// Magic bytes for vault files.
/// Fast integrity detection of wrong file type.
pub const VAULT_MAGIC: [u8; 8] = *b"OMOIDE1\0";

/// Current vault format version.
/// Increment this when the schema changes. Never change existing version behavior.
pub const VAULT_VERSION: u16 = 1;

/// Size of AES-GCM nonce in bytes.
pub const AES_NONCE_SIZE: usize = 12;

/// Size of Argon2id output in bytes.
pub const ARGON_OUTPUT_SIZE: Option<usize> = Some(32);

/// Resource limits for core dumps — set to zero to disable core dumps, preventing
/// post-mortem analysis of memory by an attacker with local access.
pub const RLIMIT_CORE_CUR: u64 = 0;

/// Resource limits for core dumps — set to zero to disable core dumps, preventing
/// post-mortem analysis of memory by an attacker with local access.
pub const RLIMIT_CORE_MAX: u64 = 0;