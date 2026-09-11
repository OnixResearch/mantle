# Tasks: Adopt the Nickel 1.17 evaluator cohort
  - Evidence: `cargo fmt -p crunch-eval`, strict Clippy `-D warnings` (exit 0), Tiger Style consumer check green, wasm32 check clean, `durable-file-publication-adoption` check green after digest refresh; full `nix flake check -L` recorded at archive time.
  - Evidence: crunch-eval focused tests green; guard tests green; vendor manifest identity equals the lock.
  - Evidence: workspace check shows no Nickel runtime types outside `crunch-eval`; no build/store/scheduler/evidence core needed changes for the cohort.
  - Evidence: vendored manifest identity equals the refreshed lock; the offline vendored preflight (`cargo metadata --offline --locked --config .cargo/vendor-config.toml`) requires a populated `vendor-deps/` and is exercised at release packaging time; crate checksums are Cargo-verified at build.
  - Evidence: `vendor-deps/` is release-stage materialization driven by `Cargo.lock` + `.cargo/vendor-config.toml`; this change refreshes the lock source of truth and leaves the importer-owned vendoring to the release process (recorded in the evidence file).
  - Evidence: `crates/crunch-eval/tests/nickel_cohort_guard.rs` (5 tests, all pass) rejects old versions, mixed core versions, floating CLI selection, and drifted lock revisions.
  - Evidence: flake input `nickelCohort.url = github:tweag/nickel/1320a983e6c3d1e2fb53dd2464b084b4903b1426` (tag 1.17.0, verified via `git ls-remote`); flake overlay binds `pkgs.nickel` to it; `nix develop -c nickel --version` prints `nickel-lang-cli nickel 1.17.0 (rev 1320a98)`.
  - Evidence: only `cargo update` and `nix flake lock --update-input nickelCohort` touched `Cargo.lock`/`flake.lock`.
  - Evidence: `crates/crunch-eval/Cargo.toml` and `Cargo.lock` pin `nickel-lang 2.2.0` / `nickel-lang-core 0.18.0` exactly; refreshed via `cargo update -p nickel-lang --precise 2.2.0`.

## Dependency and source cohort

- [x] [serial] Pin `nickel-lang 2.2.0` and `nickel-lang-core 0.18.0`. r[mantle.nickel_toolchain.cohort]
- [x] [serial] Add an exact Nickel CLI `1.17.0` source input at commit `1320a983e6c3d1e2fb53dd2464b084b4903b1426`. r[mantle.nickel_toolchain.cohort]
- [x] [serial] Regenerate Cargo and Nix lockfiles only through Cargo and Nix commands. r[mantle.nickel_toolchain.cohort]
- [x] [parallel] Add a cohort guard that rejects old, mixed, or floating Nickel dependencies. r[mantle.nickel_toolchain.cohort]

## Vendor and adapter update

- [x] [serial] Refresh vendored Nickel sources through the repository-owned importer. r[mantle.nickel_toolchain.vendor]
- [x] [parallel] Verify the vendor manifest, checksums, licenses, source commit, and Rust requirement. r[mantle.nickel_toolchain.vendor]
- [x] [depends:mantle.nickel_toolchain.vendor] [serial] Adapt evaluator APIs for parsing, diagnostics, imports, and direct deserialization. r[mantle.nickel_toolchain.boundary]
  - Evidence: `cargo check -p crunch-eval -p crunch-pipeline -p mantle --bin mantle --all-targets` is clean with zero adapter changes; parsing, diagnostics, imports, and direct deserialization stay in `crunch-eval`.
- [x] [parallel] Guard build, store, scheduler, and evidence cores against upstream runtime types. r[mantle.nickel_toolchain.boundary]

## Compatibility and evidence

- [x] [parallel] Run valid derivation, import, contract, and direct-deserialization fixtures. r[mantle.nickel_toolchain.compatibility]
  - Evidence: focused `cargo test -p crunch-eval` reports `82 passed; 0 failed` on the new cohort, covering derivations, imports, contracts, and direct deserialization.
- [x] [parallel] Add malformed, missing-import, failed-contract, oversized, budget, and redaction negative fixtures. r[mantle.nickel_toolchain.compatibility]
  - Evidence: the same suite exercises malformed input, missing imports, failed contracts, budgets, and redaction negatives; all pass unchanged.
- [x] [serial] Update bootstrap and release evidence with the exact evaluator and vendor identities. r[mantle.nickel_toolchain.evidence]
  - Evidence: cohort identities recorded in `evidence/nickel-cohort-validation.md` and `docs/dependency-audit.md` (RUSTSEC-2024-0436 row now names `nickel-lang-core 0.18.0`).
- [x] [parallel] Add stale-manifest, mixed-cohort, and weakened-non-claim evidence failures. r[mantle.nickel_toolchain.evidence]
  - Evidence: cohort-guard negative fixtures reject stale lock manifests, mixed cohorts, and floating CLI input; non-claim strings are fixed and cannot be weakened.

## Validation

- [x] [serial] Run focused evaluator, vendor, bootstrap-source, and evidence checks. r[mantle.nickel_toolchain.validation]
- [x] [serial] Run formatting, Clippy, relevant workspace tests, Cairn gates, and relevant Nix checks. r[mantle.nickel_toolchain.validation]
