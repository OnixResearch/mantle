# Implementation validation

Date: 2026-09-04

## Result

The local implementation slice passed its focused acceptance checks. Runtime tasks V2 and V3 remain open.

## Implemented boundary

- `crunch-dev-resume-core` owns bounded identity validation and contiguous-prefix selection. It is `no_std + alloc`.
- The shell owns bounded no-follow reads, BLAKE3 remeasurement, no-replace publication, restoration, execution, and reporting.
- Dev and promoted provider checkpoints use different namespaces and origins.
- Stage manifests bind source, plan, aggregate policy, stage, producer, output, execution evidence, payload, and bundle identities.
- Restore races remove partial fixed-point state and return to execution. Rejected cache identities remain in the dev report.
- `bootstrap.dev-resume-report` separates restored, executed, and newly published stage identities. It cannot claim a promoted receipt or release-alias write.

## Passing evidence

- `cargo test -p crunch-dev-resume-core --offline`: 15 passed.
- `cargo test -p mantle --bin mantle --offline source_built_fixed_point -- --nocapture`: 84 passed, 3 ignored.
- `cargo check -p crunch-dev-resume-core --target wasm32-unknown-unknown --offline`: passed.
- `cargo clippy -p mantle --bin mantle --no-deps --offline -- -D warnings`: passed.
- `cargo fmt --check -p crunch-dev-resume-core -p mantle`: passed.
- Dev-resume architecture: zero findings and nine negative fixtures.
- Machine contracts: 34 contracted and 68 classified surfaces.
- `nix build .#checks.x86_64-linux.dev-resume-core`: passed with 15 tests.
- `nix build .#checks.x86_64-linux.dev-resume-core-wasm`: passed.
- `nix build .#checks.x86_64-linux.dev-resume-architecture`: passed.
- `nix build .#checks.x86_64-linux.dev-resume-integration`: passed with 42 tests and one ignored long proof.
- Pinned Tiger Style: passed on attempt 5.
- `nix flake check --no-build --no-eval-cache`: passed on attempt 4.
- Cairn validation and proposal, design, and tasks gates: passed.
- Tracey: 157 of 157 requirements referenced.
- `git diff --check` and `git diff --cached --check`: passed.

## Preserved diagnostics

- Tiger Style attempt 1 rejected unreserved collection growth.
- Tiger Style attempt 3 rejected a quantity name without a unit suffix.
- Machine-contract attempts preserve blank freshness, unsupported schema, missing fixture-class, missing Cargo-path, and unclassified root-producer failures.
- Flake no-build attempts 1 through 3 preserve invalid local Nix derivation-path failures. Explicit Nickel and SpaceWasm realizations repaired those local paths before attempt 4 passed.
- Installed Octet first failed because `clang` was absent from `PATH`. Attempt 2 completed with status `warning-only`: 133 findings, 133 warnings, and zero errors. This is inventory evidence, not a strict pass.
- Immutable Octet revision `86ee46b3b9257b145d2dbeb6ce9d9897607db99c` remains blocked before analysis. Nix reports rust-src specified hash `sha256-q/gu/3mAuLgNfJlxV/Sw1jttbi4PIBjN+XH0bGmB5NQ=` and observed hash `sha256-WTRv7eyiu+VOfb8+90cALNJrUa3uLwRFIaeEr+tAIjQ=`.

No baseline, warning budget, disabled lint, source change, or lockfile workaround was added.
