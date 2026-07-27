# Implementation validation — 2026-07-27

## Question

Does the current implementation pass the focused code, source, format, lint, and build checks required before Cairn gates?

## Inspected evidence

- `nix develop -c cargo test -p mantle --bin mantle rust_source_provider`
  - pueue task `1177`
  - `test result: ok. 94 passed; 0 failed; 0 ignored; 0 measured; 1564 filtered out`
- `nix develop -c cargo test -p mantle --bin mantle source_toolchain_closure`
  - pueue task `1182`
  - `test result: ok. 54 passed; 0 failed; 0 ignored; 0 measured; 1604 filtered out`
- `nix develop -c cargo test -p mantle --bin mantle bootstrap_rust_source_provider`
  - pueue task `1184`
  - `test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 1646 filtered out`
- `nix develop -c cargo test -p mantle --bin mantle full_source_rust_binding`
  - pueue task `1186`
  - `test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 1615 filtered out`
- `nix develop -c cargo test -p mantle --bin mantle full_source_provider`
  - pueue task `1187`
  - `test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 1647 filtered out`
- Source-pin audit:
  - The direct command could not start because the outer environment had no `cargo`.
  - The Nix-shell Cargo is not a rustup proxy and cannot process the script's `+nightly` shebang.
  - The equivalent explicit nightly Cargo-script invocation was pueue task `1200`.
  - Result: `source-pin audit: 6 files, 3 fetch blocks, 0 issues`.
- `nix develop -c cargo fmt --check -p mantle -v`
  - pueue task `1201`
  - status `0`.
- `nix develop -c ./scripts/check-first-party-clippy.sh`
  - pueue task `1202`
  - status `0`; first-party `mantle` completed. One existing vendored `snix-castore` dead-code warning remained outside the denied first-party scope.
- `nix develop -c cargo build -p mantle --bin mantle`
  - pueue task `1203`
  - status `0`.
- `git diff --check`
  - pueue task `1205`
  - status `0`.

## Decision

The focused implementation checks pass. The source-pin script also passes when invoked with the required nightly Cargo-script runner and the six touched Nickel files.

## Owner

Mantle full-source bootstrap implementation.

## Committed-tree Cairn receipts

Implementation commit: `9e939c26`.

An initial committed-tree run passed before V3 was checked. After the task and evidence update, final pueue tasks `1232` through `1235` ran the four commands again:

- Cairn validation: `valid: true`, `issues: []`, four active changes and 37 specs validated. Full receipt: `cairn-validate-final-2026-07-27.json`.
- Proposal gate: `PASS`, receipt hash `b0fe5af9d7084e1a8920682a1dd57254ddf79fa4c84ad7f2c4fba8b7da4473ea`.
- Design gate: `PASS`, receipt hash `0aa8b931cda78f5286d6ca784aadd2ecc82c2fd220f439f72da4165a0a76d01a`.
- Tasks gate: `PASS`, seven tasks done and zero tasks remaining, receipt hash `80e859e9196d6bf05905b85b0e235ef900ac6b3d002903ea620a5ea8be50166e`.

The final gate stderr files are empty.

## Next action

Commit the completed validation packet. Then sync the accepted specification and archive this change before work starts on the StageX successor.
