## Phase 1: Spec and model

- [x] [serial] Add deterministic proof sandbox-envelope requirements and fail-closed scenarios.
- [x] [depends:spec] Extend deterministic proof receipt/schema validation with sandbox profile identity and supported executor policy.

## Phase 2: Runtime enforcement

- [x] [depends:model] Execute deterministic proof rebuild commands through the proof sandbox envelope instead of direct host process execution.
- [x] [depends:runtime] Deny network by default and reject unsupported/missing sandbox executors when deterministic proof runs are requested.
- [x] [depends:runtime] Record sandbox profile digest/evidence in deterministic proof receipts.

## Phase 3: Verification and docs

- [x] [depends:runtime] Add positive and negative tests for sandboxed deterministic proof execution, host-only recipe rejection, and missing executor failure.
- [x] [depends:runtime] Update README/docs to distinguish sandboxed deterministic proof from ordinary release reproducibility and global determinism.
- [x] [depends:verification] Run targeted Rust tests, validate OpenSpec, archive after implementation lands, commit, and push.
