## Why

Offline Cargo package outputs currently write `share/mantle/offline-cargo-build.json`, and JSON build reports surface that sidecar as `cargo_build_evidence[]`. The sidecar identifies the claim class and source/toolchain paths, but it does not strongly bind the lockfile, source/vendor digests, toolchain digests, network-denial facts, or evidence schema evolution. Malformed sidecars are ignored by the report reader, which makes diagnostics too quiet when a package intended to produce offline Cargo evidence but failed to write a valid sidecar.

Mantle should make the offline Cargo evidence sidecar digest-bound and auditable while preserving its bounded claim: Cargo ran inside Mantle's sandbox with declared offline inputs. The sidecar still must not claim Cargo-free execution, full Cargo compatibility, compiler correctness, release reproducibility, or bootstrap correctness.

## What Changes

- Introduce a versioned offline Cargo evidence schema that records lockfile digest, package source digest, optional vendor digest, toolchain digests, target/profile, Cargo command shape, network policy result, and non-claims.
- Make the build report surface valid sidecars with enough digest material to audit stale source/vendor/toolchain inputs.
- Report malformed or schema-mismatched sidecars through deterministic diagnostics when an output appears to be an offline Cargo package output.
- Keep legacy sidecar compatibility where needed, but distinguish legacy evidence from digest-bound evidence.
- Update tests and docs so offline Cargo evidence claims remain narrow and proof-before-claim compliant.

## Impact

- **Files**: `lib/offline_cargo.ncl`, `src/offline_cargo.rs`, `src/build_report.rs`, `tests/offline_cargo_project.rs`, `tests/rust_compatibility_rail.rs`, README/operator docs, and this Cairn spec delta.
- **Testing**: positive v2 sidecar/report parsing, legacy sidecar handling, malformed sidecar diagnostics, digest mismatch blockers, and non-claim wording checks.

## Out of Scope

- Claiming Cargo-free execution or replacing `rust-plan` receipts.
- Proving compiler correctness or full Cargo ecosystem compatibility.
- Requiring all existing historical build outputs to be rewritten to the new schema.
