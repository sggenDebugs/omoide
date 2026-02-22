# Omoide
An open-source desktop-only, recall-focused password manager written in Rust (known for their high memory security) with emergency recovery.

## Features
- **Safe in Hardware Memory**: Uses Rust with unsafe code for memory allocation and set.
- **Local-only**: It is never transferred to and from the desktop application.
- **Recall Mode**: Will enforce you to type the master password again under a certain time period. The more you guess your password wrong, the less time before password is entered again.
- **Emergency Recovery**: Uses optional 12-word BIP39 seed phrase (never saved to disk)
- **Auto-clear Clipboard**: copied passwords disappear after 8 seconds.
- **Encrypted Password Vault**: AES-256-GCM (Database encryption) + Argon2id (resists against GPU password cracking)

## Security Model
| Component | Implementation |
|---------|----------------|
| **Encryption** | AES-256-GCM (via `ring`) |
| **Key Derivation** | Argon2id (64 MB memory, 3 iterations, salt) |
| **Secret Erasing** | std::mem module codes in unsafe Rust |
| **Recovery** | BIP39 phrase seed |
| **Network** | Offline use |

## Status
- **Phase**: Planning
- **Outlook for Release**: June 30, 2026
- **Platform Support**: Windows, macOS, Linux
