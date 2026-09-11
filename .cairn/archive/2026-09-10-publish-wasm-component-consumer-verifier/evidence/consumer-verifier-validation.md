# Evidence: publish-wasm-component-consumer-verifier

Task-ID: mantle.wasm_consumer_verifier
Covers: contract, functional_core, file_shell, report, layers, publication, fixtures, nonclaims

## Baseline (2026-09-10)

- Internal implementation source recorded before facade changes:
  `crates/crunch-wasm-component-core/src/bundle.rs`
  (`build_materialization_bundle`, `verify_materialization_bundle`,
  `MATERIALIZATION_BUNDLE_SCHEMA`, named bounds, required stage kinds,
  required non-claims) and the standard-library verifier surfaces in
  `crates/crunch-wasm-component/src/verification.rs`.
- Focused baseline: `nix develop -c cargo check -p crunch-wasm-component -p crunch-wasm-component-core`
  finished clean on the drain branch before facade tests were added.

## Published contract

- New crate `crates/mantle-wasm-consumer-verifier`:
  - `contract.rs`: owned report DTOs, layer/status enums, no store, builder,
    release, scheduler, or CLI types; no logical paths in reports.
  - `facade.rs`: `verify_consumer_bundle` delegates every structural and
    canonical-identity decision to `crunch-wasm-component-core::
    verify_materialization_bundle`; fixed `CONSUMER_VERIFIER_NON_CLAIMS`.
  - `shell.rs` (`std` feature): `ConsumerRoot` capability-root file
    verifier with O_NOFOLLOW opens, no parent traversal, regular-file
    checks, named byte bounds, BLAKE3 remeasurement, measure-once per
    unique logical path, per-role identity judgment, `blocked` on missing
    bytes.
  - Registered in workspace members and the tigerstyle scope list.

## Consumer fixtures

- Frozen Kamacite fixture imported from
  `kamacite/fixtures/wasm-component/mantle-hello/materialization-bundle.json`
  (revision of the kamacite checkout on 2026-09-10); passes the structural
  layer and reports `blocked` over an empty capability root.
- Second-consumer world/member-graph fixture: two WIT inputs, two package
  members, AOT output bound to the final portable bytes; full verified pass.
- 12 additional positive and negative fixtures cover parity with the
  implementation, identity/schema/stage-link rejection, missing member,
  byte drift, symlink, directory member, parent traversal, oversize
  declaration, report path-leak, fixed non-claims, and no byte-layer claims
  on structural rejection.

## Verification commands and results

Command: `nix develop -c cargo test -p mantle-wasm-consumer-verifier`

    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
    test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

Command: `nix develop -c cargo clippy -p mantle-wasm-consumer-verifier --all-targets -- -D warnings`

    Finished (zero errors; `-D warnings` clean)

Command: `nix develop -c cargo fmt --check -p mantle-wasm-consumer-verifier`

    exit 0

Command: `nix develop -c cargo check -p mantle-wasm-consumer-verifier --target wasm32-unknown-unknown --no-default-features`

    Finished `dev` profile (no_std core builds for wasm32)

Command: `nix flake check -L --option builders ''` (2026-09-10, drain worktree)

    FLAKE_EXIT=0

Rerun green at final head `6d58dea6e`. An intermediate full run failed on
`rust_source_provider::tests::materializer_writes_musl_host_provider_metadata_from_route_plan`,
a third distinct pre-existing sandbox-flaky workspace bin test (module
untouched by this diff); the `crunch` check passed on immediate retry
(EXIT=0). Earlier runs also went green at `25e57797d`. Remote-builder runs
added two more unrelated host flakes; every failure was in workspace bin
tests outside this change, and all targeted checks for this crate pass at
every head.

## Non-claims

A passing report proves neither source trust, compiler correctness,
component behavior, runtime isolation, authority, reproducibility,
deployment safety, nor release eligibility. Structural-only verification is
never labeled as materialized-byte verification.
