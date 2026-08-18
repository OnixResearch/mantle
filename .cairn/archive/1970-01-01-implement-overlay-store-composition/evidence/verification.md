# Verification: implement overlay store composition

Date: 2026-08-04

## Result

The overlay implementation satisfies the selected ADR 0012 model.
One writable overlay reads through bounded, ordered, read-only bases.
All observed store paths keep exact layer and shadow evidence.

## Core and service checks

The following checks passed:

```text
cargo test --locked -p crunch-overlay-core
cargo test --locked -p crunch-gc-core
cargo test --locked -p crunch-store --lib
cargo test --locked -p crunch-pipeline
cargo test --locked -p crunch-build
cargo test --locked --test store_gc_cli
cargo test --locked --test integration_build
cargo test --locked -p snix-castore blobservice::combinator::tests
cargo test --locked -p snix-castore directoryservice::combinators::tests
cargo test --locked -p snix-store pathinfoservice::cache::tests
```

The store CLI suite passed six process tests.
It checked read-only admission, layer reporting, no base mutation, plan-bound GC, and root migration.

The filesystem overlay tests checked the following cases:

- Base-only PathInfo, directory, file, and blob reads.
- No PathInfo, directory, or blob backfill.
- Overlay and ordered base precedence.
- Matching, conflicting, colliding, and failed shadows.
- Base action-result discovery without base publication.
- CA mapping precedence with overlay-only publication.
- Cryptographic per-layer PathInfo trust.
- Writable, stale, corrupt, incomplete, duplicate, and mismatched bases.
- Overlay-only GC and base-to-overlay reference rejection.

A real Bubblewrap build fixture now consumes a base-only source.
This host has no usable `bwrap`, so the fixture recorded its bounded skip path.
The composed service and filesystem tests exercised the same input services without that host dependency.

## Reports and contracts

The following producer checks passed:

```text
cargo test --locked --bin mantle build_report::tests::build_json_report_includes_artifact_attestation_reference
cargo test --locked --bin mantle realization_routing::tests::route_report_
cargo test --locked --bin mantle attest_cmd::tests::attestation_envelope_
cargo test --locked --bin mantle machine_contract_producer_tests
```

Route reports bind the overlay plan and selected base descriptors.
Build reports bind base generations, selected layers, and shadows.
Attestation envelopes report the selected artifact or closure layers.

A focused run of the repository generator updated the three changed contracts.
It reported `machine schema contract generation: PASS (23 contracted, 53 classified)`.

The canonical generator self-test passed:

```text
cargo -Zscript scripts/check-machine-schema-contracts.rs --self-test
```

The full contract check still stops at unrelated inventory gaps.
The first gaps are in `src/content_bound_requirement_evidence.rs` and `src/full_source_rust_binding.rs`.
The changed producers and registered positive fixtures passed their focused tests.

## Policy and quality checks

The typed Nickel default matched its generated JSON.
The positive fixture exported successfully.
All invalid overlay fixtures failed during export.

The following focused quality checks passed:

```text
cargo fmt --all --check
cargo clippy --locked -p crunch-overlay-core -p crunch-gc-core --all-targets -- -D warnings
cargo clippy --locked --no-deps -p crunch-store -p crunch-build -p crunch-pipeline --all-targets -- -D warnings
cargo check --locked -Zbuild-std=core,alloc --target wasm32-unknown-unknown -p crunch-overlay-core
git diff --check
```

Dependency-inclusive Clippy still fails in vendored `fuse-backend-rs`.
The first failures are unrelated `useless_conversion` and `io_other_error` findings.

The full Mantle binary test run reached 2,097 passing tests and 66 ignored tests.
Four unrelated tests failed.
Two failures are stale bootstrap evidence digests.
One failure needs a fake Slurm process.
One concurrent remote-build lock test exhausted its retries.

## Cairn checks

The cached Cairn binary passed these checks:

```text
cairn validate --root .
cairn gate proposal implement-overlay-store-composition --root .
cairn gate design implement-overlay-store-composition --root .
cairn gate tasks implement-overlay-store-composition --root .
cairn traceability coverage --root . --json
```

Traceability reported 155 referenced requirements, no missing references, and no dangling references.

## External blockers

`nix flake check path:$PWD -L` cannot fetch this required input:

```text
https://git.onix.computer/z3tAR4For7qw8ZirkJzoDw1VNDDLM.git
```

Nix reports that the repository does not exist.
This blocker occurs before the new overlay flake check can run.

Broad vendored Snix tests that need FUSE remain blocked by the missing host descriptor device.
Focused blob, directory, and PathInfo service tests passed.

## Bounded claims

This evidence does not prove permanent base immutability.
It does not prove whole-database atomicity, source correctness, or release eligibility.
It does not grant mutation authority over a base.
It binds results only to the observed descriptors, generations, trust keys, and selected layers.
