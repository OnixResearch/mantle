# Focused validation of the remote-build hexagon (2026-11-03)

Scope: evidence for I1-I6 and V1-V4. V5's full `nix flake check -L` could not be
scheduled on this host (see the end).

## Implementation evidence

| Task | Evidence |
| --- | --- |
| I1 | `crates/crunch-remote-core` and `crates/crunch-remote-app` open with `#![no_std]`, `#![forbid(unsafe_code)]`, `extern crate alloc`; the app manifest depends only on `crunch-remote-core`. |
| I2 | `crunch-remote-core` owns admission, attempt/fencing, effect, identity, lease, output, pool, protocol, receipt, recovery, resource, resource_policy, resource_quota, transfer, endpoint, external_batch, failure, nominal, provider, caveat, and demand projection modules. |
| I3 | `crunch-remote-app` exposes application ports: executor, external batch, transport, client exchange, attempt persistence, attempt transition, lease, input transfer, output transfer, store admission, credential verification, clock observation, telemetry, plus the effect session. Adapters live in `src/remote_build*` and `crates/crunch-build/src/distributed*`. |
| I4 | The application contracts use Mantle-owned values; the app manifest has no Snix/Tokio/filesystem/process/environment/clock dependency, and Snix `PathInfo`/build request/result values are projected only in adapters. |
| I5 | `crunch_remote_app::execute` drives bounded typed effects through ports and records each `Observation`/`ObservedEffect` before the next transition; mismatched observations surface `EffectBlocker::WrongObservationKind`. |
| I6 | Gateway (`src/remote_build/gateway.rs`), resource policy/quota, nominal, and Trellis projection modules remain with their owners and tests; this change only adapts around them. |

## Test evidence

- `cargo test -p crunch-remote-core -p crunch-remote-app -- --test-threads 1`:
  core 119 passed (includes 115 core unit tests), app 10 passed.
- `cargo test -p crunch-build distributed -- --test-threads 1`: 125 passed.
- `cargo test -p mantle --bin mantle remote_build:: -- --test-threads 1` with the
  sandbox environment: 269 passed.
- `tests/remote_hexagon_v2_parity.rs`: 7 tests —
  accepted stdio wire/receipt match, executable-plan preimage match, malformed
  frame diagnostic, physical signed output/fenced transfer, upload quota,
  untrusted key, failed-adapter semantics.
- `cargo check -p crunch-remote-core --target wasm32-unknown-unknown`: exit 0.
- `cargo clippy -p crunch-remote-core -p crunch-remote-app -p crunch-build -p mantle --all-targets --no-deps -- -D warnings`: exit 0.
- `git diff --check`: exit 0.
- Cairn `gate proposal|design|tasks separate-remote-build-hexagon`: all `PASS`.

## V5 limitation

`nix flake check -L` could not complete on this host: the daemon accumulated over
1300 worker processes and the check waited in evaluation/GC contention until
cancelled. `nix flake check --no-build -L` evaluates all checks ("all checks
passed"). The full build check remains unevidenced; V5 stays open.
