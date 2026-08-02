# Verification evidence

Date: 2026-08-02

The repository used this current external Cairn policy:

`/home/brittonr/git/OnixResearch/cairn/cairn-policy/generated/cairn-policy.json`

The repository-local generated policy lacks `nominal_identity_policy`.

## Focused checks

| Command | Result |
|---|---|
| `nix develop -c cargo fmt --all -- --check` | pass |
| `nix develop -c cargo test -p mantle --bin mantle foreign_` | pass |
| `nix develop -c cargo test -p mantle --bin mantle source_bundle` | pass |
| `nix develop -c cargo test -p mantle --bin mantle build_correctness` | pass |
| `nix develop -c cargo test -p crunch-store --lib` | pass |
| `nix develop -c cargo test -p crunch-pipeline --lib` | pass |
| `nix develop -c cargo test -p crunch-glue --lib` | pass |
| `nix develop -c cargo test -p mantle --test foreign_import_cli` | pass |
| `nix develop -c cargo check -p mantle --bin mantle` | pass |
| `nix develop -c cargo build -p mantle --bin mantle` | pass |
| `nix develop -c cargo check --workspace --all-targets` | pass |
| `nix develop -c cargo clippy -p crunch-store --lib --no-deps -- -D warnings` | pass |
| `nix develop -c cargo clippy -p crunch-pipeline --lib --no-deps -- -D warnings` | pass |
| `nix develop -c cargo clippy -p crunch-glue --lib --no-deps -- -D warnings` | pass |
| `nix develop -c cargo clippy -p mantle --bin mantle --test foreign_import_cli --no-deps -- -D warnings` | pass after a named audit-closure type removed one complexity finding |
| `nix develop -c check-nickel-configs` | pass, including three expected cache-policy rejections |
| `nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs` | pass |
| `nix develop -c cargo -Zscript scripts/check-foreign-import-trust-model.rs --self-test` | pass |
| `nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs --self-test` | pass |
| `git diff --check` | pass |

The local HTTP tests include positive and negative cache closure cases.
They verify metadata preflight before NAR transfer and PathInfo mutation.
The scheduler test verifies removal of all builder dependencies and arguments.

## Broad baseline result

The extra broad binary test ran 1,973 active tests.
It passed 1,970 and failed three unrelated environment-sensitive tests.
It also ignored 65 tests.

The failures were two bootstrap parity tests and one Rust source-provider test.
The source-provider log reports missing `dirname` in its synthetic `PATH`.
The focused foreign and changed-package suites all passed.

## Machine inventory baseline

The machine-schema checker retains 39 unrelated StageX and source-build inventory findings.
It reports no foreign import, cache closure, realization, or provenance source finding.

## Cairn pre-sync status

Strict validation passed.
Proposal, design, and tasks gates returned `PASS` and `valid: true`.
The tasks gate observed 14 completed tasks and one pending V3 task before this evidence was recorded.

Pre-sync Tracey reported 266 referenced requirements and 693 accepted requirements.
Unrelated repository-wide missing and dangling references kept `valid: false`.

## Live proof

See `live-nixpkgs-hello/summary.md` for the realized, reuse, audit, pull, and negative identities.
All Mantle consumption steps used a `PATH` without Nix.
