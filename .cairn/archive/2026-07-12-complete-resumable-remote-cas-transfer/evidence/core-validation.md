# Resumable transfer functional-core validation

Date: 2026-07-12
Requirements:
- `r[remote_builds.attempt_scoped_transfer_resume]`
- `r[store_transports.resumable_castore_sessions]`
- `r[store_transports.receiver_driven_backpressure]`
- `r[store_transports.content_presence_early_cutoff]`

## Implementation boundary

`crates/crunch-build/src/distributed/remote_transfer.rs` owns the pure deterministic core. It defines bounded typed session/attempt/fence, manifest, artifact/chunk, demand, credit, acknowledgement, checkpoint, receiver-fact, and completion DTOs. It canonicalizes and BLAKE3-identifies manifests/policies/checkpoints, recomputes receiver demand from verified facts, validates resume without trusting cursors, uses checked quota arithmetic, and keeps output admission as an explicit non-claim.

The manifest artifact classes are the existing Mantle boundaries: castore blob/directory, NAR, source bundle, PathInfo, attestation, delta blob, and delta chunk. No storage service or second CAS is defined in the core.

## Current checks

Pueue task 376:

```text
cargo test -p crunch-build --lib distributed::remote_transfer::tests:: -- --nocapture

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 527 filtered out; finished in 0.02s
```

Coverage includes canonical permutation invariance across every artifact class, deterministic missing sets, current-checkpoint resume, stale fence, wrong/tampered checkpoint digest, regressed acknowledgements, acknowledged-but-missing receiver content, forged cursor, credit exhaustion before payload allocation, chunk digest mismatch, monotonic acknowledgement, idle-progress bounds, and early-cutoff rejection for expected-identity-only or unadmitted PathInfo facts. Proptest covers checked quota arithmetic, monotonic acknowledgement sets, and equivalent-fact demand determinism.

Pueue task 381:

```text
rustfmt --check: PASS
cargo test -p crunch-build --lib distributed::remote_transfer::tests:: -- --nocapture: PASS
cargo check -p crunch-build --lib: PASS (existing crunch-store warning baseline remains)
remote-transfer-core-purity: PASS
 git diff --check: PASS
```

The purity scan rejected filesystem, environment, process, printing, clock, Tokio, and async APIs in the new core.

Three `#[cfg(kani)]` source harnesses cover checked-add overflow behavior, credit rejection without state mutation, and the expected-identity cutoff boundary. Pueue task 390 recorded `cargo-kani: unavailable`, so this evidence does not claim Kani execution.

## Non-claims

This evidence proves the pure decision core and its focused tests only. It does not yet prove shell streaming, durable checkpoint I/O, production frame replacement, interruption/restart across processes, output admission, or fallback behavior.
