# Threat Model
> **Last Updated**: February 10, 2026 \
> **Goal**: Protect master password, decrypted secrets, and recovery seed.

**STRIDE** Model is used.

## Assets to Protect
| Asset | Protection Goal |
|------|-----------------|
| Master password | Never stored; never logged; wiped from memory after use |
| Decrypted passwords | Only in memory; zeroized immediately after use |
| Vault file (`vault.db`) | Encrypted at rest; resistant to offline brute-force |
| BIP39 recovery seed | Shown once; never persisted; zeroized after 60s |

---

## Top Threats & Mitigations
### 1. **Master password or decrypted secrets exposed in memory dumps files**
- **STRIDE Category**: Information Disclosure
- **Risk**: Core dumps, swap files, or debuggers expose decrypted passwords.
- **Mitigation**:
  - Use `zeroize` crate to wipe all secret types on `Drop`
  - Disable core dumps at runtime (`libc::setrlimit under unsafe Rust`) 
  - Avoid logging any sensitive data (even in debug builds)

### 2. **Offline Brute-Force Attack on Vault File**
- **Risk**: Weak KDF allows GPU cracking of master password.
- **Mitigation**:
  - Use **Argon2id** with high memory cost (≥64 MB, 3 iterations)
  - Enforce strong master password (entropy ≥ 60 bits)
  - No fallback to PBKDF2

### 3. **Recovery Seed Accidentally Persisted**
- **Risk**: Seed phrase saved to disk, logs, or clipboard.
- **Mitigation**:
  - Generate seed **only in memory**
  - Never write to file, console, or error messages
  - Zeroize after 60 seconds or when window loses focus

### 4. **"Recall Mode" Bypassed via Side Channels**
- **Risk**: Screen capture, clipboard snooping, or shoulder surfing.
- **Mitigation**:
  - Auto-clear clipboard after 8 seconds
  - Mask passwords until prefix is typed correctly
  - Use OS-level secure window flags (prevent screenshots)

### 5. **Malicious or Vulnerable Dependencies**
- **Risk**: Compromised crate (e.g., fake `ring`) steals secrets.
- **Mitigation**:
  - Pin exact versions in `Cargo.lock`
  - Run `cargo audit` weekly
  - Prefer audited crates (`ring` is FIPS 140-2 compliant)

---

## Non-Negotiable Security Rules (v1)
- **No network access**: Remove all HTTP clients; block outbound traffic during testing
- **All secrets zeroized**: Every `Password`, `MasterKey`, and `SeedPhrase` implements `Zeroize`
- **Recovery requires 2 factors**: Seed phrase + master password hash
- **No telemetry**: Not even crash reporting

---

## References
- [OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
- [NIST SP 800-63B: Digital Identity Guidelines](https://pages.nist.gov/800-63-3/)
