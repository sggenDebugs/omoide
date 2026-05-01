# Phase 2: Recall Mode SRS State Schema

## Implemented Design (Current)

> [!NOTE]
> The proposal below ("Store Inside the Encrypted Entry Object") was the original design
> intent. It was **superseded** during implementation. See the "Accepted Risk" section for
> the rationale. The code in `omoide-format` reflects the implemented design, not the
> original proposal.

### Where SRS State Lives

SRS state is stored in the **plaintext `VaultHeader`**, not inside the encrypted `Entry`
object. The `VaultHeader` carries an `SrsState` field serialised as part of the CBOR
vault file, before the encrypted entries block.

```rust
// omoide-format/src/schema.rs
pub struct VaultHeader {
    pub kdf_params: KdfParams,
    pub salt: [u8; KDF_SALT_SIZE],
    pub header_aad: [u8; HEADER_AAD_SIZE],
    pub srs_state: SrsState,   // <-- plaintext; readable before any decryption
}

pub struct SrsState {
    /// Current interval in hours before the next Emergency Access Rehearsal (EAR).
    pub current_interval_hours: f32,

    /// Unix timestamp (seconds) of the last successful EAR.
    pub last_rehearsal: u64,

    /// Count of consecutive EAR failures since the last success.
    pub consecutive_failures: u8,
}
```

### Why Plaintext Header (Design Rationale)

The orchestrator (`omoide-core/src/orchestrator.rs`) must check **before asking for the
master password** whether a reprompt is due. This requires reading the SRS deadline
(`last_rehearsal + current_interval_hours * 3600`) without having the derived key in
memory.

If SRS state were inside the encrypted `Entry`, the vault would have to be fully
decrypted just to answer "is a rehearsal due right now?" — which defeats the purpose of
the EAR gating the decryption.

### Accepted Risk — Usage Pattern Side Channel

Storing SRS state in the plaintext header creates a **usage pattern side channel**:

- An attacker with **read-only access to the vault file** can observe:
  - The current interval length (a proxy for how recently the user rehearsed).
  - The `last_rehearsal` Unix timestamp.
  - The `consecutive_failures` count.
- The attacker **cannot** learn any entry content; all passwords, usernames, and notes
  remain encrypted under AES-256-GCM.

**Threat category**: Information Disclosure (STRIDE) — metadata only.  
**Accepted**: Yes. The operational benefit (correct EAR gating without pre-decryption)
outweighs the metadata leakage for the target threat model (local vault file theft).
This risk is documented in `THREAT_MODEL.md` under Threat 4.

---

## Superseded Proposal: Store Inside the Encrypted `Entry` Object

> **Status: Superseded.** Retained for historical context only. Not implemented.

The original proposal was to track a `StrikeCounter` per entry inside the encrypted
`Entry` CBOR structure to prevent any usage timing side-channel attacks.

```rust
// SUPERSEDED — not implemented

#[derive(Debug, Serialize, Deserialize, ZeroizeOnDrop)]
pub struct Entry {
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub notes: String,

    // --> NEW PHASE 2 FIELDS <--
    pub strike_state: StrikeCounter,

    pub created: u64, // Unix timestamp
    pub updated: u64, // Unix timestamp
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct StrikeCounter {
    /// Increments on correct answer. Resets/decreases on incorrect.
    pub consecutive_successes: u32,

    /// Total count of incorrect answers (never resets)
    pub lifetime_failures: u32,

    /// The next Unix timestamp when this entry must be reprompted.
    pub next_prompt_timestamp: u64,

    /// Dynamic interval modifier, akin to an Anki "ease" factor.
    pub ease_factor: f32,
}
```

This design was rejected because the reprompt deadline must be evaluated before
decryption occurs. A vault-wide SRS state in the plaintext header is the pragmatic
solution; the per-entry strike counter design may be revisited in a future phase if
per-entry SRS granularity becomes necessary.
