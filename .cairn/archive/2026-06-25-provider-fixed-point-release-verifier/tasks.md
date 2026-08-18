# Tasks

- [x] [serial] I1 Add fixed-point proof bundle validation core and shell loading for existing `meta.json` / `preflight.json` / stage artifacts. r[rust_package_planning.provider_fixed_point_release_verifier]
- [x] [serial] I2 Thread optional and required provider fixed-point proof verification through `mantle release verify` human and JSON output. r[rust_package_planning.provider_fixed_point_release_verifier]
- [x] [serial] V1 Add positive and negative tests for valid bundle evidence, stage digest mismatch, missing enforced closure, and missing non-claims. r[rust_package_planning.provider_fixed_point_release_verifier]
  - Evidence: `evidence/final-validation-2026-06-25.md` records focused cargo-free verifier, release command, and CLI parse tests.
- [x] [serial] V2 Run focused Rust tests, build, diff check, Cairn validation/gates, and record evidence. r[rust_package_planning.provider_fixed_point_release_verifier]
  - Evidence: `evidence/final-validation-2026-06-25.md` records build, diff check, Cairn validate, and proposal/design/tasks gates.
