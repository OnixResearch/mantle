# Store capability migration evidence

## Oracle checkpoint

- **Question:** Does Mantle complete the store capability migration without changing store identity, trust, admission, publication order, or public report meaning?
- **Inspected evidence:** `crunch-store` now owns concrete transfer, administration, attestation, provenance, source, foreign-realization, Rust-cache, build-service, lookup, root, and publication capabilities. The AST architecture rail reports zero external runtime findings. Its positive and negative fixtures pass. Compile-fail doctests pass seven cases. Store, Rust-cache, and delta suites pass 360, 25, and 37 unit tests, plus two authority tests and seven delta integration tests. Pipeline suites pass 40 unit and 19 integration tests, with four ignored tests unchanged. Remote-transfer tests pass 18 main tests and two subprocess tests. Workspace all-target compilation, strict first-party Clippy, formatting, `git diff --check`, the exact Tiger Style Nix gate, the dedicated architecture Nix gate, the durable-publication adoption gate, and `nix flake check --no-build -L` pass.
- **Decision:** Accept the implementation for committed-source review. Runtime callers use role-specific capabilities. Local output admission returns a BLAKE3-bound publication plan before any publisher runs. Publisher attempts return typed observations. Store formats, signatures, logical paths, action-result records, build-report JSON, and operator command meaning remain unchanged.
- **Owner:** Mantle maintainers.
- **Next action:** Commit the implementation, rerun committed-source lifecycle gates, record the bounded full-check result, synchronize the accepted requirement, and archive the change.

## Negative evidence

The tests reject broad-handle access, raw service escape, unrelated build authority, writable base-service authority, malformed capability requests, missing store facts, publisher failure, and tampered effect identity. A publisher failure occurs only after local admission and does not erase admitted `PathInfo` truth.

## Claim boundary

This evidence proves bounded API reachability, source-policy conformance, effect ordering, and the listed tests. It does not prove store content correctness, publisher honesty, sandbox isolation, filesystem confinement, remote availability, reproducibility, or release eligibility.
