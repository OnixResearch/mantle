# Evidence: adopt-nickel-1-17-cohort

Task-ID: mantle.nickel_toolchain
Covers: cohort, vendor, boundary, compatibility, evidence, validation

## Cohort identity (2026-09-10)

- Embedded: `nickel-lang 2.2.0`, `nickel-lang-core 0.18.0` (crates.io,
  pinned exact in `crates/crunch-eval/Cargo.toml` and `Cargo.lock`).
- CLI: Nickel `1.17.0` at upstream commit
  `1320a983e6c3d1e2fb53dd2464b084b4903b1426` (tag `1.17.0`, verified with
  `git ls-remote`), pinned as flake input `nickelCohort` and enforced over
  every former `pkgs.nickel` use through a flake overlay.
- Rust requirement: the cohort builds with the workspace toolchain
  unchanged (`rust-toolchain.toml` untouched).

## Vendor process

- Lockfiles regenerated only through `cargo update -p nickel-lang
  --precise 2.2.0` and `nix flake lock --update-input nickelCohort`; no
  hand edits to normalized vendored manifests.
- `vendor-deps/` is release-stage input materialization, not a checked-in
  tree; the cohort's vendored crates flow through the repository-owned
  release vendoring driven by `Cargo.lock` and
  `.cargo/vendor-config.toml`. The lock is the source of truth updated
  here.

## Boundary

- `cargo check -p crunch-eval -p crunch-pipeline -p mantle --bin mantle
  --all-targets`: zero errors with no adapter changes; parsing, direct
  deserialization, diagnostic rendering, and budgets stay in
  `crunch-eval`. No upstream Nickel runtime types leaked into other cores.

## Compatibility fixtures

- Focused `cargo test -p crunch-eval`: `test result: ok. 82 passed` —
  positive derivation evaluation, imports, contracts, direct
  deserialization, and negative malformed-input, missing-import,
  failed-contract, budget, and redaction fixtures all pass on the new
  cohort without semantic changes.
- New `tests/nickel_cohort_guard.rs`: five tests proving the committed
  cohort is exact and rejecting old embedded versions, mixed
  `nickel-lang-core` versions, floating CLI selection, and drifted
  `nickelCohort` lock revisions.

## Evidence records

- `docs/dependency-audit.md` RUSTSEC-2024-0436 row updated to
  `nickel-lang-core 0.18.0` (paste remains an upstream dependency).
- CLI identity proven live: `nix develop -c nickel --version` prints
  `nickel-lang-cli nickel 1.17.0 (rev 1320a98)`.

## Non-claims

Evaluation success on the new cohort proves neither evaluator correctness,
Nix equivalence, derivation correctness, build hermeticity, fixed-point
reproducibility, nor output trust.
