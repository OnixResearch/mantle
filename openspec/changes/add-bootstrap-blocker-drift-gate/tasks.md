## Phase 1: Spec foundation

- [x] [serial] Define blocker inventory/drift-gate requirements and validation scope.

## Phase 2: Checker implementation

- [ ] [serial] Implement the bootstrap blocker inventory checker with stable marker classes and JSON/Markdown report output.
- [ ] [parallel] Add positive evidence for the current gated tree showing blockers are inventoried without claiming promotion.
- [ ] [parallel] Add negative fixture or mutation evidence proving promotion claims fail while blockers remain.
- [ ] [depends:checker] Wire the checker into the appropriate bootstrap/readiness validation rail without making ordinary edit-time checks expensive.

## Phase 3: Documentation and closeout

- [ ] [depends:checker] Document the report contract, marker taxonomy, and retirement workflow for repaired blockers.
- [ ] [depends:checker] Run strict OpenSpec validation, checker evidence, whitespace checks, and archive once all implementation evidence is captured.
