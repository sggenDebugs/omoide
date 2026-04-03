// Usage: cargo run --release -p omoide-crypto --bin benchmark_kdf
//
// Run with --release — debug builds are not representative of real performance.

use omoide_crypto::kdf::{derive_master_key, KdfParams};
use std::time::Instant;

fn bench(label: &str, params: &KdfParams) {
    let password = b"benchmark-password";
    let salt = b"benchmark_salt_32bytesxxxxxxxxxx";

    let start = Instant::now();
    derive_master_key(password, salt, params).expect("KDF failed");
    let elapsed = start.elapsed();

    println!("{label:30} => {:>8.1?}", elapsed);
}

fn main() {
    println!("omoide bench-kdf — Argon2id parameter survey");
    println!("Target: ~300ms on this machine\n");

    // OWASP configurations from your threat model
    bench("m=19456 t=2 p=1 (default)", &KdfParams {
        memory_cost: 19456, time_cost: 2, parallelism_cost: 1,
    });
    bench("m=47104 t=1 p=1", &KdfParams {
        memory_cost: 47104, time_cost: 1, parallelism_cost: 1,
    });
    bench("m=12288 t=3 p=1", &KdfParams {
        memory_cost: 12288, time_cost: 3, parallelism_cost: 1,
    });
    bench("m=9216  t=4 p=1", &KdfParams {
        memory_cost: 9216,  time_cost: 4, parallelism_cost: 1,
    });
    bench("m=7168  t=5 p=1", &KdfParams {
        memory_cost: 7168,  time_cost: 5, parallelism_cost: 1,
    });

    println!("\nRecord results in ADR-001 before finalizing vault format.");
}