# Transfer shell validation

Date: 2026-07-12

## Implemented boundary

- `src/remote_transfer.rs` is the imperative shell over the pure transfer core.
- Production adapters cover existing castore blob/directory identities, NAR files, source bundles, PathInfo, canonical attestation bytes, and existing delta blob/chunk/PathInfo frames. No new CAS was added.
- `mantle-remote-transfer-data-frame-v1` separates a bounded JSON control header from one credit-reserved payload chunk and works over socket/stdio-compatible `Read`/`Write` streams.
- Durable state atomically fsyncs a checkpoint plus lease under `state_dir/remote-transfers/<session>.json`; reconnect re-probes receiver files, and stale-scope/fence or expired leases are invalidated.
- Upload and download use the same receiver-demanded quota/credit path. `lib/remote-builders.ncl` now exports typed endpoint `transfer_policy`; `src/remote_farm_config.rs` deserializes and validates the same `RemoteTransferPolicy` used by the runtime shell.
- Runtime streaming reports can only be constructed from a completed `RemoteTransferShellReport`. Capability negotiation alone no longer causes `plan_output_transfer()` to claim streaming. Whole-payload NAR/PathInfo DTOs are documented and retained only as compatibility fallback.
- Delta-to-full-NAR fallback has stable reasons and still passes through digest, closure/PathInfo, and separate output-admission facts.

## Focused evidence

Pueue task 691:

```text
cargo test -p mantle --bin mantle remote_transfer::tests:: -- --nocapture

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1374 filtered out; finished in 0.21s
```

This includes positive multi-chunk upload/download interruption, an 8 MiB bounded local output rail, current-fence resume, process restart, no duplicate resend, complete-content zero-byte cutoff, socket data framing, and delta/full fallback. Negative coverage includes tampered acknowledged chunks, stale fence scope, expired leases, missing admission, oversized control/total/inline inputs, and digest/identity rejection. The child-process rails each also reported `1 passed; 0 failed`.

Pueue task 673:

```text
cargo test -p mantle --bin mantle remote_farm_config::tests:: -- --nocapture

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1372 filtered out; finished in 0.04s
```

This includes positive/default typed policy evaluation and negative Nickel type mismatch plus Rust hard-limit validation.

Pueue task 663:

```text
cargo test -p mantle --bin mantle remote_build::tests:: -- --nocapture

test result: ok. 104 passed; 0 failed; 0 ignored; 0 measured; 1282 filtered out; finished in 1.01s
```

This confirms the compatibility full-NAR/delta fallback, ordinary input/output digest checks, remote output admission, durable coordinator fencing, and operator stdio rail remain intact.

## Non-claims

- Transfer completion does not claim output admission; every runtime report sets `output_admission_claimed = false`.
- The compatibility whole-payload fallback DTOs do not prove streaming.
- Kani harness execution remains unclaimed because `cargo-kani` was unavailable; source harness evidence remains in `evidence/core-validation.md`.
