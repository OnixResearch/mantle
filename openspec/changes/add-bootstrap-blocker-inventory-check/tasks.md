## Phase 1: Implementation

- [x] [serial] Inspect `scripts/check-bootstrap-blocker-inventory.rs` and choose the smallest maintained invocation surface. ✅ 1m (started: 2026-05-12T03:14:48Z → completed: 2026-05-12T03:15:26Z)
  Evidence: chose the existing maintained wrapper `./scripts/check-bootstrap-blocker-inventory.sh`; `bash -n` passed, and wrapper report-only/self-test passed with Nix clang/toolchain PATH (309 findings across 6 classes, 0 promotion claims).
- [x] [serial] Add or update the deterministic check/fixture so known findings are categorized. ✅ 1m (started: 2026-05-12T03:15:55Z → completed: 2026-05-12T03:16:44Z)
  Evidence: expanded `run_self_tests()` fixtures to exercise all six marker classes; `./scripts/check-bootstrap-blocker-inventory.sh --self-test --report-only` passed and reported all six classes.
- [x] [serial] Run the inventory check and `git diff --check`, then record evidence. ✅ 1m (started: 2026-05-12T03:17:01Z → completed: 2026-05-12T03:17:26Z)
  Evidence: `./scripts/check-bootstrap-blocker-inventory.sh --self-test` passed in enforcement mode (309 findings across 6 classes, 0 promotion claims); `git diff --check` passed.

## Phase 2: Completion

- [x] [depends:implementation] Run `openspec validate add-bootstrap-blocker-inventory-check --strict` and record evidence before archive. ✅ 1m (started: 2026-05-12T03:17:43Z → completed: 2026-05-12T03:17:59Z)
  Evidence: `openspec validate add-bootstrap-blocker-inventory-check --strict` passed.
