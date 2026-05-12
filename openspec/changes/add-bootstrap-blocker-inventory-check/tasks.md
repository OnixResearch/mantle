## Phase 1: Implementation

- [x] [serial] Inspect `scripts/check-bootstrap-blocker-inventory.rs` and choose the smallest maintained invocation surface. ✅ 1m (started: 2026-05-12T03:14:48Z → completed: 2026-05-12T03:15:26Z)
  Evidence: chose the existing maintained wrapper `./scripts/check-bootstrap-blocker-inventory.sh`; `bash -n` passed, and wrapper report-only/self-test passed with Nix clang/toolchain PATH (309 findings across 6 classes, 0 promotion claims).
- [ ] [serial] Add or update the deterministic check/fixture so known findings are categorized.
- [ ] [serial] Run the inventory check and `git diff --check`, then record evidence.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate add-bootstrap-blocker-inventory-check --strict` and record evidence before archive.
