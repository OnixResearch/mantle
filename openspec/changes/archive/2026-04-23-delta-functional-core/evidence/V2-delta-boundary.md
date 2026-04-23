Evidence-ID: delta-functional-core-v2-delta-boundary
Task-ID: V2
Artifact-Type: verification
Covers: architecture.nostd.core.crate.boundary.effectful.dependency.outside, functional.core.apis.plain.data.typed.results.normalized.request.no.ambient.reads, functional.core.dedicated.nostd.crates.third.wave.semantic.parity, functional.core.shell.adapters.effect.translation.delta.substitution.io.in.shell, functional.core.shell.adapters.effect.translation.delta.facade.compatibility
Status: complete
Date: 2026-04-22

# V2 verification packet

## Command transcript

Environment used for cargo runs:

- `CARGO_TARGET_DIR=/tmp/crunch-delta-validate-target`
- `SNIX_BUILD_SANDBOX_SHELL=/bin/sh`
- `PATH` prefixed with nightly Rust + clang + mold + pkg-config wrappers
- `PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig`

Executed commands and observed results:

1. `cargo test -p crunch-delta-core chunk_profile_v1_matches_spec`
   - `test model::tests::chunk_profile_v1_matches_spec ... ok`
2. `cargo test -p crunch-delta-core chunk_profile_wire_v1_matches_runtime_profile`
   - `test negotiation::tests::chunk_profile_wire_v1_matches_runtime_profile ... ok`
3. `cargo test -p crunch-delta-core prefix_mismatch_is_rejected`
   - `test planner::tests::prefix_mismatch_is_rejected ... ok`
4. `cargo test -p crunch-delta-core planner_matches_fixed_suite_targets`
   - `test planner::tests::planner_matches_fixed_suite_targets ... ok`
5. `cargo test -p crunch-delta-core duplicate_version_offer_is_rejected`
   - `test negotiation::tests::duplicate_version_offer_is_rejected ... ok`
6. `cargo test -p crunch-delta substitution_adapter_keeps_async_store_and_network_in_shell`
   - `test substitution::tests::substitution_adapter_keeps_async_store_and_network_in_shell ... ok`
7. `cargo test -p crunch-delta delta_facade_reexports_core_planner_types`
   - `test planner::tests::delta_facade_reexports_core_planner_types ... ok`
8. Supplemental std-facade compatibility checks:
   - `cargo test -p crunch-delta plan_error_keeps_std_error_in_facade`
     - `test planner::tests::plan_error_keeps_std_error_in_facade ... ok`
   - `cargo test -p crunch-delta negotiation_error_keeps_std_error_in_facade`
     - `test negotiation::tests::negotiation_error_keeps_std_error_in_facade ... ok`
9. Full suite spot-checks after the façade restore:
   - `cargo test -p crunch-delta-core`
     - `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
   - `cargo test -p crunch-delta`
     - `test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`

## Source inspection

### `crunch-delta-core`

- `crates/crunch-delta-core/src/lib.rs` remains `#![no_std]` with `extern crate alloc` and only exports model/negotiation/planner logic.
- `crates/crunch-delta-core/src/model.rs` keeps the normalized core boundary:
  - `DeltaDigest([u8; 32])`
  - `ReceiverManifest` membership sets as `BTreeSet<DeltaDigest>`
  - deterministic `Vec` traversal for outputs/children/chunks
- `crates/crunch-delta-core/src/negotiation.rs` and `crates/crunch-delta-core/src/planner.rs` keep typed `NegotiationError` / `PlanError` with `Display` only; no `std::error::Error` impls live in the core crate.

### `crunch-delta` std façade

- `crates/crunch-delta/src/lib.rs` exports the std-facing façade directly; it no longer publishes `pub use crunch_delta_core as core;`.
- `crates/crunch-delta/src/model.rs` restores the std-facing boundary on runtime-shaped types:
  - public `ReceiverManifest` uses `HashSet<B3Digest>`
  - public node/fixture digests are `B3Digest`
  - conversion into normalized core requests happens in helper functions such as `core_receiver_manifest(...)` and `core_closure_fixture(...)`
- `crates/crunch-delta/src/negotiation.rs` restores façade-owned `NegotiationOffer`, `NegotiatedProtocol`, `ChunkProfileWire`, and `NegotiationError`, including `impl std::error::Error for NegotiationError`, while delegating actual negotiation to `crunch-delta-core`.
- `crates/crunch-delta/src/planner.rs` restores façade-owned `PlanError` with `impl std::error::Error for PlanError`, converts sender/receiver façades into normalized core requests, and maps the core `TransferPlan` back into the std-facing type.
- `crates/crunch-delta/src/fixtures.rs` remains std-owned fixture-building code.
- `crates/crunch-delta/src/manifest.rs` remains std-owned castore probing / manifest construction.
- `crates/crunch-delta/src/substitution.rs` remains std-owned async store/network/session/attestation shell code.

## Verdict

V2 satisfied on 2026-04-22.

The core crate still owns the normalized no-std protocol/planning logic, while the std-facing `crunch-delta` façade now again contains the runtime-shaped conversion boundary and the std `Error` integration that the design explicitly reserved for the adaptor layer.
