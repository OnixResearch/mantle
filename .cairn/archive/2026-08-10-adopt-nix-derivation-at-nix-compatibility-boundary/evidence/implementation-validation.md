# Nix derivation adoption validation

## Result

The reviewed adapter passed its focused positive and negative tests.
The exact package, source, license, and upstream revision checks also passed.

## Focused checks

The exact transcript is in `final-focused-validation.log`.

- The adapter passed 7 tests.
- The foreign import boundary passed 29 tests.
- The Nix producer core passed 28 tests.
- The Nix producer shell passed 8 tests.
- `nix-derivation` passed 52 unit, integration, corpus, and documentation tests.
- The boundary guard passed its self-test and repository scan.
- Targeted Rust formatting and `git diff --check` passed.
- Focused first-party Clippy passed with the existing large-enum allowance.

The source-parity transcript is in `source-admission-validation.log`.
The offline vendor check is in `vendor-offline-validation.log`.

## Positive evidence

- One exact dependency uses crate version `0.1.0`.
- One production adapter imports `nix_derivation`.
- Standard `/nix/store` ATerm imports use the reviewed adapter.
- The existing Guix prefix rewrite keeps the prior parser.
- The adapter preserves arbitrary environment bytes before projection.
- It derives the logical name before native identity calculation.
- It classifies all output and dynamic-input forms.
- It enforces named byte, field, collection, edge, and depth limits.
- Covered fixtures retain their expected Nix SHA-256 paths and identities.
- Direct and directory imports read candidate files through bounded shell functions.

## Negative evidence

The adapter rejects unsupported forms before graph or publication effects.
Coverage includes malformed input, limits, missing names, non-UTF-8 fields, and structured-attribute projection failures.
Coverage also includes versioned, dynamic, floating, deferred, impure, text, and Git forms.

The guard rejects dependency drift, lock drift, adapter drift, unapproved imports, and source parity drift.
Its negative self-tests cover each finding class.

## Broad repository checks

`cargo test --workspace` ran to completion and recorded 2,292 passes in the root binary.
It stopped with five failures in untouched bootstrap, Slurm, and seccomp modules.
Three deterministic bootstrap failures reproduce on `origin/main` at `465e4b45`.
See `workspace-tests.log` and `workspace-main-baseline-reproduction.log`.

`cargo clippy --workspace --all-targets -- -D warnings` stopped in vendored `fuse-backend-rs`.
The focused first-party Clippy command passed.
See `workspace-clippy.log` and `focused-clippy.log`.

`cargo fmt --all -- --check` found one unrelated difference in `src/source_built_fixed_point_shell.rs`.
Targeted formatting for every changed Rust file passed.
See `fmt-all.log` and `final-focused-validation.log`.

The Nix package check fetched the exact package and compiled the release binary.
It reached 173 library test passes, then hit the known seccomp sandbox test failure.
See `nix-package-check.log`.

## Cairn and Tracey

The locked Cairn CLI passed strict validation and the proposal, design, and tasks gates.
Review completeness passed without findings.
The accepted requirements do not appear in the missing or dangling Tracey lists.
Repository-wide Tracey still reports existing debt outside this change.
See `cairn-active-validation.log` and `cairn-post-sync-validation.log`.

## Claim boundary

This evidence proves only the checked Nix compatibility metadata boundary.
It does not prove builder safety, sandboxing, derivation correctness, output correctness, reproducibility, or release eligibility.
