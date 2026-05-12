## Phase 1: Provider-kind linkage receipt

- [x] [serial] Add a checked `crunch.self-build` provider-kind linkage evidence contract to the parity report.
  - Evidence: `src/bootstrap_parity.rs` validates `bootstrap/evidence/crunch-self-build-provider-kind-linkage.json` with schema `crunch-self-build-provider-kind-linkage-v1`, closed provider-kind values, and equality across `proof_identity.selected_provider_kind`, `proof_linkage.selected_provider_kind`, and `prerequisites.provider_kind`.
- [x] [depends:contract] Add positive and negative parity tests for matching, missing, unknown, and mismatched provider-kind receipt data.
  - Evidence: `cargo test --bin crunch bootstrap_parity::tests -- --nocapture` passed 26 tests at 2026-05-12T20:08:30Z, including matching, missing, unknown, mismatched, and real-repo receipt coverage.
- [x] [depends:tests] Run targeted parity tests, parity-report JSON, OpenSpec validation, and whitespace checks.
  - Evidence: `cargo fmt --check`, targeted parity tests, `cargo run -q -p crunch -- --json bootstrap parity-report`, `openspec validate check-self-build-provider-kind-linkage --strict`, `openspec validate --all --strict` (49 passed), and `git diff --check` passed.
- [x] [depends:verification] Sync/archive this OpenSpec change after verification.
  - Evidence: ready for `openspec archive check-self-build-provider-kind-linkage --yes` after the verified implementation and task transcript above.
