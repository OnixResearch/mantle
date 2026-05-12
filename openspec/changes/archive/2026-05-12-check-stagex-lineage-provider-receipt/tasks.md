## Phase 1: StageX lineage receipt

- [x] [serial] Add a checked StageX lineage provider receipt contract to the parity report.
  - Evidence: `src/bootstrap_parity.rs` validates `bootstrap/evidence/stagex-lineage-provider-receipt.json` with schema `crunch-stagex-lineage-provider-receipt-v1`, `provider_kind = stagex-lineage`, explicit `lineage_receipt_status = scaffold-only`, digest-shaped lineage fields, and empty fallback events.
- [x] [depends:contract] Add positive and negative parity tests for missing, matching scaffold, wrong provider kind, malformed digest, and fallback events.
  - Evidence: `cargo test --bin crunch bootstrap_parity::tests -- --nocapture` passed 32 tests at 2026-05-12T20:24:16Z, including the StageX lineage receipt cases and real-repo partial-status regression.
- [x] [depends:tests] Run targeted parity tests, parity-report JSON, OpenSpec validation, and whitespace checks.
  - Evidence: `cargo fmt --check`, targeted parity tests, `cargo run -q -p crunch -- --json bootstrap parity-report`, `openspec validate check-stagex-lineage-provider-receipt --strict`, `openspec validate --all --strict` (49 passed), and `git diff --check` passed.
- [x] [depends:verification] Sync/archive this OpenSpec change after verification.
  - Evidence: ready for `openspec archive check-stagex-lineage-provider-receipt --yes` after the verified implementation and task transcript above.
