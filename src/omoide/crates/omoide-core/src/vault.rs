use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use omoide_env::VAULT_MAGIC;
use omoide_format::schema::{EncryptedEntry, VaultFile, VaultHeader};

use crate::error::VaultError;

/// Open a vault file, read and validate its format, and return the `VaultFile` layout.
/// This does not decrypt the entries.
pub fn open(path: &Path) -> Result<VaultFile, VaultError> {
    let mut file = File::open(path)?;

    // Read and validate magic bytes
    let mut magic = [0u8; VAULT_MAGIC.len()];
    file.read_exact(&mut magic).map_err(|e| {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            VaultError::InvalidMagic
        } else {
            VaultError::Io(e)
        }
    })?;
    if magic != VAULT_MAGIC {
        return Err(VaultError::InvalidMagic);
    }

    // Read header
    let header: VaultHeader = ciborium::de::from_reader(&mut file)?;

    // Read entries
    // Uses standard from_reader logic for the CBOR array of entries
    let entries: Vec<EncryptedEntry> = ciborium::de::from_reader(&mut file)?;

    Ok(VaultFile { header, entries })
}

/// Serialize and save the `VaultFile` layout to the given path using an atomic swap.
pub fn seal(vault: &VaultFile, path: &Path) -> Result<(), VaultError> {
    let tmp_path = path.with_extension("omoide.tmp");
    let mut tmp_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&tmp_path)?;

    // Write magic bytes
    tmp_file.write_all(&VAULT_MAGIC)?;

    // Serialize header
    ciborium::ser::into_writer(&vault.header, &mut tmp_file)?;

    // Serialize entries
    ciborium::ser::into_writer(&vault.entries, &mut tmp_file)?;

    // Perform fsync for data durability
    tmp_file.sync_all()?;

    // At this point we can safely close the file by dropping tmp_file implicitly
    drop(tmp_file);

    #[cfg(windows)]
    eprintln!("Warning: Filesystem rename may not be completely atomic natively on Windows");

    // Atomic filesystem rename
    std::fs::rename(&tmp_path, path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use omoide_crypto::kdf::KdfParams;
    use tempfile::tempdir;

    fn dummy_vault() -> VaultFile {
        VaultFile {
            header: VaultHeader {
                kdf_params: KdfParams::default(),
                salt: [0; 32],
                header_aad: [0; 32],
            },
            entries: vec![],
        }
    }

    #[test]
    fn test_open_seal_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("vault.omoide");
        let vault = dummy_vault();

        // Write the empty VaultFile
        seal(&vault, &path).expect("failed to seal");

        // Read it back
        let opened = open(&path).expect("failed to open");
        assert_eq!(vault.header.salt, opened.header.salt);
        assert_eq!(vault.entries.len(), opened.entries.len());
    }

    #[test]
    fn test_invalid_magic_bytes() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("invalid.omoide");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"NOTMAGIC").unwrap();

        let res = open(&path);
        assert!(matches!(res, Err(VaultError::InvalidMagic)));
    }
}
