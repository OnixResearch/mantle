## Phase 1: Spec and probe implementation

- [x] [serial] Validate the v17 OpenSpec change before implementation.
- [x] [serial] Add compact `ssize_t` macro-vs-typedef/order diagnostic probes.
- [x] [serial] Update v17 evidence schema, observed frontier, diagnostic markers, and parity checks.
- [x] [serial] Update the bootstrap spec with bounded v17 and stale-evidence scenarios.

## Phase 2: Verification and landing

- [x] [serial] Run focused diagnostic/parity verification and CLI parity-report checks proving `gcc.4.0` remains `partial`.
- [x] [serial] Run source-pin/blocker-inventory self-tests, `openspec validate --all --strict`, and `git diff --check`.
- [x] [serial] Archive the change, commit, push, and confirm clean synced state.
