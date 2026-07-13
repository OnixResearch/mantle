# Transfer shell validation

Date: 2026-07-12

## Implemented boundary

- `src/remote_transfer.rs` is the imperative shell over the pure transfer core.
- Production adapters cover existing castore blob/directory identities, NAR files, source bundles, PathInfo, canonical attestation bytes, and existing delta blob/chunk/PathInfo frames. No new CAS was added.
- `mantle-remote-transfer-data-frame-v1` separates a bounded JSON control header from one credit-reserved payload chunk and works over socket/stdio-compatible `Read`/`Write` streams.
- Durable state atomically fsyncs a checkpoint plus lease under `state_dir/remote-transfers/<session>.json`; a per-session exclusive lock rejects concurrent writers, reconnect re-probes receiver files, and stale-scope/fence or expired leases are invalidated.
- Upload and download use the same receiver-demanded quota/credit path. `lib/remote-builders.ncl` now exports typed endpoint `transfer_policy`; `src/remote_farm_config.rs` deserializes and validates the same `RemoteTransferPolicy` used by the runtime shell.
- Runtime streaming reports can only be constructed from a completed `RemoteTransferShellReport`. Capability negotiation alone no longer causes `plan_output_transfer()` to claim streaming.
- The bounded shell exposes deterministic delta-to-full-NAR fallback reasons and keeps transfer completion separate from caller-supplied admission facts.
- The production stdio/ssh-stdio state machine now carries bounded transfer manifests, receiver demand, one-chunk credits, data frames, acknowledgements, and completion around the existing build request/result control frames. Production NARs and PathInfo are spooled to files and ingested through ordinary signed output admission; whole inline payloads remain bounded fixture/bootstrap compatibility only.
- Production dispatch holds an exclusive coordinator mutation guard across the fenced transfer/admission operation, validates the current attempt before and after durable writes and before completion, and invalidates stale attempt-scoped receiver/checkpoint facts before acknowledgement. Authority-bearing checkpoint, lock, chunk, artifact, temp, and parent-sync paths reject symlink/non-regular races and cap checkpoint reads at the hard limit plus one byte.

## Focused evidence

Pueue task 719:

```text
cargo test -p mantle --bin mantle remote_transfer::tests:: -- --nocapture

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 1374 filtered out; finished in 0.25s
```

This includes positive multi-chunk upload/download interruption in the standalone shell, an 8 MiB bounded local shell rail, current-fence resume, process restart, no duplicate resend, complete-content zero-byte cutoff, socket data framing helpers, and standalone delta/full fallback. Negative coverage includes a forged canonical manifest, a concurrent same-session writer, tampered acknowledged chunks, stale fence scope, expired leases, missing admission facts, oversized control/total/inline inputs, and digest/identity rejection. The child-process rails each also reported `1 passed; 0 failed`. None of these tests routes the payload through the production `run_stdio_remote_child` / `cmd_remote_serve` framed protocol.

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

## Production path evidence

Pueue task 1017 ran `remote_build::tests::`, `remote_transfer::tests::`, and the initial `remote_transfer_production` target in one isolated `&&`-chained packet. It reported 109 remote-build tests, 17 transfer-shell tests plus two successful subprocess legs, and 3 production integration tests, all passing. Those production tests prove multi-chunk upload/download interruption and missing-only restart, output reused-byte accounting, ordinary store admission, and fail-closed upload quota.

Pueue task 1140 then ran current `crunch-store --lib`, `crunch-delta --lib`, and five production integration tests. The 199, 36, and 5 tests all passed. The added public `--remote-delta` rail proves stable `delta-unavailable` fallback to full-NAR bounded chunks and ordinary admission; the added 8 MiB rail proves more than 100 acknowledged chunks before byte-identical admission. Task 1166 independently reran those packages together with all 109 remote-build tests. Exact commands and result lines are in `evidence/final-validation.md`.

## Non-claims

- Transfer completion does not itself claim output admission; production admission is a later ordinary signed PathInfo/store step asserted separately by the integration tests.
- `delta-unavailable` is an honest fallback reason: the test proves both peers advertise delta but no production delta runtime is bound, so it does not claim a delta hit.
- Kani harness execution remains unclaimed because `cargo-kani` was unavailable; source harness evidence remains in `evidence/core-validation.md`.
