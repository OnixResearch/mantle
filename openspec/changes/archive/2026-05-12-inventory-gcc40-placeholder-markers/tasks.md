## Phase 1: GCC 4.0 placeholder inventory

- [x] [serial] Add a checked `gcc.4.0` placeholder inventory receipt and parity validation.
  - Evidence: `src/bootstrap_parity.rs` validates `bootstrap/evidence/gcc-4.0-placeholder-inventory.json` with schema `crunch-gcc40-placeholder-inventory-v1`, derivation `bootstrap/gcc-4.0.ncl`, status `inventory-only`, exact marker count, and exact line/marker occurrences.
- [x] [depends:receipt] Add positive and negative parity tests for matching, missing, and drifted inventories.
  - Evidence: `cargo test --bin crunch bootstrap_parity::tests -- --nocapture` passed 36 tests at 2026-05-12T20:43:12Z, including matching/missing/drifted GCC 4.0 inventory cases and a real-repo partial-status regression.
- [x] [depends:tests] Run targeted parity tests, parity-report JSON, OpenSpec validation, and whitespace checks.
  - Evidence: `cargo fmt --check`, targeted parity tests, `cargo run -q -p crunch -- --json bootstrap parity-report`, `openspec validate inventory-gcc40-placeholder-markers --strict`, `openspec validate --all --strict` (49 passed), and `git diff --check` passed. Parity report now shows `gcc.4.0 partial unknown failed=False` while keeping it in live-bootstrap blockers.
- [x] [depends:verification] Archive and commit the completed change.
  - Evidence: ready for `openspec archive inventory-gcc40-placeholder-markers --yes` after the verified implementation and task transcript above.
