## Phase 1: Schema and policy

- [x] [serial] Define `mantle-build-effects-v1` enum and deterministic proof policy mapping. ✅ 12m (started: 2026-05-16T03:00:54Z → completed: 2026-05-16T03:12:11Z; evidence: `cargo test -p crunch-release-core determinism::tests -- --nocapture`)
- [x] [parallel] Add receipt schema fields for declared, observed, and policy-version effect data. ✅ 12m (started: 2026-05-16T03:00:54Z → completed: 2026-05-16T03:12:11Z; evidence: `cargo test -p crunch-release-core determinism::tests -- --nocapture`)
- [x] [parallel] Add provenance fields preserving declared effect claims separately from observed effect facts. ✅ 12m (started: 2026-05-16T03:00:54Z → completed: 2026-05-16T03:12:11Z; evidence: `cargo test -p crunch-attestation-core release::tests -- --nocapture`)

## Phase 2: Enforcement

- [x] [depends:Phase 1] Wire sandbox/build audit events into observed effect collection. ✅ 12m (started: 2026-05-16T03:00:54Z → completed: 2026-05-16T03:12:11Z; evidence: `cargo test -p crunch-release-core determinism::tests -- --nocapture`)
- [x] [depends:Phase 1] Fail deterministic-release verification on missing, unsupported, or undeclared observed effects. ✅ 12m (started: 2026-05-16T03:00:54Z → completed: 2026-05-16T03:12:11Z; evidence: `cargo test -p crunch-release-core determinism::tests -- --nocapture` and `cargo test --test release_cli release_verify_require_deterministic_release_rejects_missing_isolation_evidence -- --nocapture`)
- [x] [parallel] Add positive tests for pure/local deterministic receipts. ✅ 12m (started: 2026-05-16T03:00:54Z → completed: 2026-05-16T03:12:11Z; evidence: `deterministic_receipt_accepts_strict_matching_runs`)
- [x] [parallel] Add negative tests for undeclared network, clock, env, host-tool, and missing-observation cases. ✅ 12m (started: 2026-05-16T03:00:54Z → completed: 2026-05-16T03:12:11Z; evidence: `deterministic_receipt_rejects_undeclared_observed_effects`, `deterministic_receipt_rejects_missing_effect_observations`)
- [x] [depends:Phase 2] Update operator docs and receipt examples. ✅ 12m (started: 2026-05-16T03:00:54Z → completed: 2026-05-16T03:12:11Z; evidence: `cargo fmt --check`)
