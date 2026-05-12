## Phase 1: Implementation

- [ ] [serial] Inspect `scripts/check-bootstrap-blocker-inventory.rs` and choose the smallest maintained invocation surface.
- [ ] [serial] Add or update the deterministic check/fixture so known findings are categorized.
- [ ] [serial] Run the inventory check and `git diff --check`, then record evidence.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate add-bootstrap-blocker-inventory-check --strict` and record evidence before archive.
