## Phase 1: Baseline and low-risk structural repairs

- [x] [serial] I1 [covers=store_lifecycle.tiger_conformance] [evidence=evidence/validation.md] Preserve the focused 183-finding and repository 139-finding baselines, including file and lint-family counts. r[store_lifecycle.tiger_conformance]
  - Evidence: `evidence/baseline-2026-09-01/` records both failing strict commands, all 13 file counts, and 359 passing pre-change tests.
- [x] [serial] I2 [covers=store_lifecycle.tiger_conformance] Repair explicit-unit names, checked collection reservations, and compound conditions without changing rejection behavior. r[store_lifecycle.tiger_conformance]
  - Evidence: focused Tiger Style reports no `crunch-store` finding after explicit-unit, bounded-growth, checked-arithmetic, and condition repairs. Negative tests retain their typed errors.
- [x] [serial] I3 [covers=store_lifecycle.tiger_conformance] Replace ambiguous private parameter clusters with named input records while preserving public interfaces. r[store_lifecycle.tiger_conformance]
  - Evidence: private helpers use named request records. The public HTTP validator uses `HttpClosureImportValidation`, and the Mantle caller compiles against that explicit contract.

## Phase 2: Core and shell decomposition

- [x] [serial] I4 [covers=store_lifecycle.tiger_conformance] Split GC, root, retention, overlay, and composition functions along existing observation, decision, and mutation phases. r[store_lifecycle.tiger_conformance]
  - Evidence: GC keeps plan-before-mutation order, composition uses an explicit bounded stack, and root/overlay tests pass before and after the split.
- [x] [serial] I5 [covers=store_lifecycle.tiger_conformance] Split Nario, pull, HTTP-closure, query, handle, layer, and capability functions without reordering external effects. r[store_lifecycle.tiger_conformance]
  - Evidence: Nario publication, HTTP closure admission, overlay layer selection, and bounded listing tests pass with the same fail-closed outcomes.
- [x] [serial] I6 [covers=store_lifecycle.tiger_conformance] Split provenance scanning and container parsing into bounded deterministic helpers with explicit invariants. r[store_lifecycle.tiger_conformance]
  - Evidence: 21 focused provenance tests cover accepted payloads, malformed containers, limits, path escape, trust failure, and deterministic classification.

## Phase 3: Verification and lifecycle

- [x] [serial] V1 [covers=store_lifecycle.tiger_conformance] [evidence=evidence/validation.md] Run positive and negative `crunch-store` tests before and after the core changes. r[store_lifecycle.tiger_conformance]
  - Evidence: both baselines pass 357 unit and 2 authority-policy integration tests with zero failures.
- [x] [serial] V2 [covers=store_lifecycle.tiger_conformance] [evidence=evidence/validation.md] Run focused and repository Tiger Style to zero `crunch-store` findings, then run strict Clippy, formatting, and diff checks without allowances. r[store_lifecycle.tiger_conformance]
  - Evidence: both Tiger transcripts contain zero `crunch-store` finding. Strict Clippy, formatting, Mantle caller compilation, and no-build Nix evaluation pass. The repository Tiger command reaches the later 20-finding build boundary.
- [x] [serial] V3 [covers=store_lifecycle.tiger_conformance] [evidence=evidence/validation.md] Run local and ordinary full flake checks, preserve any later exact blocker, then validate, sync, archive, commit, push, and integrate. r[store_lifecycle.tiger_conformance]
  - Evidence: local and ordinary full checks contain no `crunch-store` finding and preserve the later build-family Tiger diagnostics. `evidence/store-repair-2026-09-01/summary.md` records scope and non-claims.
