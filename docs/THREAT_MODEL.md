# Threat Model
> **Last Updated**: May 2, 2026 \
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
  - **Use `zeroize`** crate to wipe all secret types on `Drop`. Prevents exposure *after* deallocation but does not prevent swap to disk.
  - **Swap/hibernation** — Pin secret memory pages with `mlock` (Linux/macOS)
    and `VirtualLock` (Windows) via the `memsec` crate before populating them
    with plaintext. This prevents the OS from paging secret memory to disk
    entirely. Apply to: master password buffer, derived vault key, any
    decrypted entry held in memory.
  - **Disable core dumps at runtime** (`libc::setrlimit under unsafe Rust`)
  - Avoid logging any sensitive data (even in debug and trace builds). The
    `secrecy` crate (`Secret<T>` with `[REDACTED]` debug output) is planned
    for a future phase but is not yet active in the current codebase.

### 2. **Offline Brute-Force Attack on Vault File**
- **STRIDE Category**: Information Disclosure
- **Risk**: Weak KDF allows GPU cracking of master password.
- **Mitigation**:
  - Use **Argon2id** with high memory cost (≥19 MB, 2 iterations) *refer to [KDF Parameter Rationale](#kdf-parameter-rationale)
  - Enforce strong master password (entropy ≥ 60 bits)
  - No fallback to PBKDF2

### 3. **Recovery Seed Accidentally Persisted**
- **STRIDE Category**: Information Disclosure
- **Risk**: Seed phrase saved to disk, logs, or clipboard.
- **Mitigation**:
  - Generate seed **only in memory**
  - Never write to file, console, or error messages
  - Zeroize after 60 seconds or when window loses focus

### 4. **"Recall Mode" Bypassed via Side Channels**
- **STRIDE Category**: Information Disclosure, Elevation of Privilege
- **Risk**: Screen capture, clipboard snooping, or shoulder surfing.
- **Mitigation**:
  - Auto-clear clipboard after 8 seconds
  - Mask passwords until prefix is typed correctly
  - Use OS-level secure window flags (prevent screenshots)

### 5. **Malicious or Vulnerable Dependencies**
- **STRIDE Category**: Tampering
- **Risk**: Compromised crate (e.g., fake `ring`) steals secrets.
- **Mitigation**:
  - Pin exact versions in `Cargo.lock`
  - Run `cargo audit` weekly
  - Prefer audited crates

---

## KDF Parameter Rationale
**Chosen:** `m=19456` (19 MiB), `t=2`, `p=1` — OWASP recommended balanced 
baseline. Provides GPU brute-force resistance without excessive unlock latency 
on modest hardware. Benchmark on your machine with `omoide bench-kdf` and 
tune via `config.toml` if needed.

**OWASP alternative configurations** (all are acceptable):
* m=47104 (46 MiB), t=1, p=1 (Do not use with Argon2i)
* m=12288 (12 MiB), t=3, p=1
* m=9216 (9 MiB), t=4, p=1
* m=7168 (7 MiB), t=5, p=1

where m - minimum memory size, t - iterations, p - parallelism

---

## Non-Negotiable Security Rules (v1)
- **No network access**: Remove all HTTP clients; block outbound traffic during testing
- **All secrets zeroized**: Every `Password`, `MasterKey`, and `SeedPhrase` implements `Zeroize`
- **Recovery**: Seed phrase only derives vault key
- **No telemetry**: Not even crash reporting

---

## References
- [OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html)
- [NIST SP 800-63B: Digital Identity Guidelines](https://pages.nist.gov/800-63-3/)
