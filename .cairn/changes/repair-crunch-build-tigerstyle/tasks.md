## Phase 1: Baseline and request normalization

- [x] [serial] I1 [covers=build_correctness.tiger_conformance] [evidence=evidence/validation.md] Preserve the focused 20-finding baseline by file and lint family, plus passing pre-change positive and negative package tests. r[build_correctness.tiger_conformance]
  - Evidence: `evidence/baseline-2026-09-01/` records 20 strict findings across six files and 702 passing pre-change tests.
- [ ] [serial] I2 [covers=build_correctness.tiger_conformance] Repair checked bounds, quantity names, named build-request inputs, and structured-output validation without changing accepted environment or output behavior. r[build_correctness.tiger_conformance]
- [ ] [serial] I3 [covers=build_correctness.tiger_conformance] Replace recursive structured-attribute canonicalization and production `expect` calls with bounded iterative processing and typed failures while preserving canonical bytes. r[build_correctness.tiger_conformance]

## Phase 2: Planning, profiles, orchestration, and publication

- [ ] [serial] I4 [covers=build_correctness.tiger_conformance] Remove content-addressed planning panic paths without changing valid output selection, derivation names, or output names. r[build_correctness.tiger_conformance]
- [ ] [serial] I5 [covers=build_correctness.tiger_conformance] Repair execution-profile assertions, condition shape, predicate names, and path-validation interfaces while preserving accepted and rejected profiles. r[build_correctness.tiger_conformance]
- [ ] [serial] I6 [covers=build_correctness.tiger_conformance] Introduce explicit registry and worker request records without changing insertion, failure propagation, waiter notification, or root outcomes. r[build_correctness.tiger_conformance]
- [ ] [serial] I7 [covers=build_correctness.tiger_conformance] Add meaningful no-replace publication invariants in `crunch-rustc-wrapper` without changing filesystem authority or error behavior. r[build_correctness.tiger_conformance]

## Phase 3: Verification and lifecycle

- [ ] [serial] V1 [covers=build_correctness.tiger_conformance] [evidence=evidence/validation.md] Run `nix develop -c cargo test -p crunch-build -p crunch-rustc-wrapper --lib --tests` before and after core changes, with positive and negative coverage. r[build_correctness.tiger_conformance]
- [ ] [serial] V2 [covers=build_correctness.tiger_conformance] [evidence=evidence/validation.md] Run `nix run .#tigerstyle -- check -- -p crunch-build -p crunch-rustc-wrapper`, repository Tiger Style, strict Clippy, formatting, and diff checks without allowances. r[build_correctness.tiger_conformance]
- [ ] [serial] V3 [covers=build_correctness.tiger_conformance] [evidence=evidence/validation.md] Run local and ordinary full flake checks, preserve any later exact blocker, then validate, sync, archive, commit, push, and integrate. r[build_correctness.tiger_conformance]
