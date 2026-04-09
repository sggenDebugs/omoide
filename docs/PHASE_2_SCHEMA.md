# Phase 2: Recall Mode Strike Counter Schema Proposal

## The spaced-repetition logic (`Strike Counter`)
To facilitate active recall enforcement without compromising the vault's privacy, `omoide` will track a `StrikeCounter` for each entry.

### Proposal: Store Inside the Encrypted `Entry` Object
To prevent usage timing side-channel attacks, space-repetition intervals and metrics **must not** be stored in the plaintext `VaultHeader`. 

We will modify the core `Entry` CBOR structure inside `omoide-format` as follows:

```rust
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
    /// If memory-resident and this timestamp is reached, the entry transitions
    /// to `locked_awaiting_reprompt` and is zeroed.
    pub next_prompt_timestamp: u64,

    /// Dynamic interval modifier, akin to an Anki "ease" factor.
    pub ease_factor: f32, 
}
```

### Flow & Persistence
- Every time an entry is fetched or displayed after its `next_prompt_timestamp`, a re-auth prompt is raised.
- **Success:** `consecutive_successes` ++. Calculate new interval using `ease_factor`. Set `next_prompt_timestamp = now + new_interval`. Write to vault file using atomic swap.
- **Failure:** `consecutive_successes` is zeroed. `lifetime_failures` ++. `ease_factor` decreases. The entry drops out of memory immediately. Next interval is halved.
- **Vault Save:** Because `Entry` is modified, the specific `EncryptedEntry` is re-encrypted with a fresh OS-level Nonce, keeping the ciphertext indistinguishable.
