## Phase 1: Baseline and low-risk structural repairs

- [x] [serial] I1 [covers=store_lifecycle.tiger_conformance] [evidence=evidence/validation.md] Preserve the focused 183-finding and repository 139-finding baselines, including file and lint-family counts. r[store_lifecycle.tiger_conformance]
  - Evidence: `evidence/baseline-2026-09-01/` records both failing strict commands, all 13 file counts, and 359 passing pre-change tests.
- [ ] [serial] I2 [covers=store_lifecycle.tiger_conformance] Repair explicit-unit names, checked collection reservations, and compound conditions without changing rejection behavior. r[store_lifecycle.tiger_conformance]
- [ ] [serial] I3 [covers=store_lifecycle.tiger_conformance] Replace ambiguous private parameter clusters with named input records while preserving public interfaces. r[store_lifecycle.tiger_conformance]

## Phase 2: Core and shell decomposition

- [ ] [serial] I4 [covers=store_lifecycle.tiger_conformance] Split GC, root, retention, overlay, and composition functions along existing observation, decision, and mutation phases. r[store_lifecycle.tiger_conformance]
- [ ] [serial] I5 [covers=store_lifecycle.tiger_conformance] Split Nario, pull, HTTP-closure, query, handle, layer, and capability functions without reordering external effects. r[store_lifecycle.tiger_conformance]
- [ ] [serial] I6 [covers=store_lifecycle.tiger_conformance] Split provenance scanning and container parsing into bounded deterministic helpers with explicit invariants. r[store_lifecycle.tiger_conformance]

## Phase 3: Verification and lifecycle

- [ ] [serial] V1 [covers=store_lifecycle.tiger_conformance] [evidence=evidence/validation.md] Run positive and negative `crunch-store` tests before and after the core changes. r[store_lifecycle.tiger_conformance]
- [ ] [serial] V2 [covers=store_lifecycle.tiger_conformance] [evidence=evidence/validation.md] Run focused Tiger Style to zero findings, repository Tiger Style, strict Clippy, formatting, and diff checks without allowances. r[store_lifecycle.tiger_conformance]
- [ ] [serial] V3 [covers=store_lifecycle.tiger_conformance] [evidence=evidence/validation.md] Run local and ordinary full flake checks, preserve any later exact blocker, then validate, sync, archive, commit, push, and integrate. r[store_lifecycle.tiger_conformance]
