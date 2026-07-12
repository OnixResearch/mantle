## Phase 1: Final decision core

- [x] [serial] Define normalized release verification contributor results, selected policy requirements, and a pure final decision with closed disposition and ordered diagnostics. r[mantle.release_provenance.verification_decision.complete] r[mantle.release_provenance.verification_decision.boundary]
  - Evidence: `crates/crunch-release-core/src/verification_decision.rs`; host and wasm validation are recorded in `evidence/validation.md`.
- [x] [serial] Make contributor coverage exhaustive and add a focused completeness check for newly required evidence policies. r[mantle.release_provenance.verification_decision.completeness] r[mantle.release_provenance.verification_decision.completeness.test]
  - Evidence: the single-source contributor macro and `completeness_guard_maps_every_required_contributor`; see `evidence/validation.md`.
- [x] [serial] Add plain pure-core assertions for optional absence, full acceptance, and one or multiple required-policy blockers. r[mantle.release_provenance.verification_decision.boundary.test]
  - Evidence: six pure decision tests passed within the 150-test core run in pueue task 1086; see `evidence/validation.md`.

## Phase 2: CLI ordering and rendering

- [x] [serial] Refactor `cmd_release_verify` to collect normalized facts, aggregate the complete decision, and only then invoke a renderer or select process status. r[mantle.release_provenance.verification_decision.complete]
  - Evidence: `src/release_cmd.rs::{evaluate_release_verification,aggregate_release_verify_decision,emit_release_verification}`.
- [x] [serial] Render the human success marker only for an accepted final decision and use explicit non-success wording for rejection. r[mantle.operator_diagnostics.release_verification.terminal_verdict]
  - Evidence: human positive/negative CLI fixtures passed in pueue tasks 1058 and 1059.
- [x] [serial] Version JSON output with top-level `valid`, closed `disposition`, ordered `checks`, and ordered `diagnostics` for acceptance and policy rejection. r[mantle.operator_diagnostics.release_verification.json_contract]
  - Evidence: `mantle-release-verify-v2` rendering and migrated receipt consumers passed in pueue tasks 1058 and 1201.
- [x] [serial] Keep rendering effect-free over the immutable decision and reserve JSON stdout for exactly one payload. r[mantle.operator_diagnostics.release_verification.render_boundary] r[mantle.operator_diagnostics.release_verification.render_boundary.test]
  - Evidence: renderer immutability/terminal-verdict unit test passed in pueue task 1094; JSON rejection fixtures parse exactly one stdout value.

## Phase 3: Positive and negative fixtures

- [x] [parallel] Add human and JSON success fixtures proving the success marker appears only after every selected check passes. r[mantle.release_provenance.verification_decision.fixtures.positive] r[mantle.operator_diagnostics.release_verification.fixtures.positive]
  - Evidence: positive human ordering and accepted JSON checks passed in pueue task 1058.
- [x] [parallel] Add independent required-reproducibility, deterministic-proof, provider-proof, stack-provenance, external-role, and StageX rejection fixtures. r[mantle.release_provenance.verification_decision.fixtures.negative]
  - Evidence: independent late-gate fixtures passed in pueue tasks 1058 and 1059; case mapping is recorded in `evidence/validation.md`.
- [x] [parallel] Assert every human rejection exits nonzero without `release evidence verified` or equivalent success wording. r[mantle.operator_diagnostics.release_verification.fixtures.negative]
  - Evidence: shared `assert_human_release_verify_rejection` covers all six active late gates.
- [x] [parallel] Assert every JSON policy rejection emits one parseable `valid: false` result, ordered diagnostics, and a nonzero exit. r[mantle.operator_diagnostics.release_verification.json_negative]
  - Evidence: shared `assert_json_release_verify_rejection` verifies status, one JSON payload, closed rejection disposition, and flattened diagnostic order.

## Phase 4: Validation and documentation

- [x] [serial] Document the final validity/disposition contract and migration rule for JSON consumers. r[mantle.operator_diagnostics.release_verification.json_contract]
  - Evidence: `README.md` and `docs/operator-workflows.md` document v2 and require both zero status and `valid: true`.
- [x] [serial] Run focused release command unit tests and release CLI positive/negative output tests. r[mantle.release_provenance.verification_decision.fixtures.positive] r[mantle.release_provenance.verification_decision.fixtures.negative]
  - Evidence: pueue tasks 1086, 1094, 1058, 1059, 1181, 1201, and 1099 are summarized in `evidence/validation.md`.
- [x] [serial] Run Cairn validation and proposal, design, and tasks gates before sync/archive. r[mantle.release_provenance.verification_decision.complete]
  - Evidence: pueue tasks 1114, 1120, 1124, and 1125 passed; final post-evidence reruns are appended to `evidence/validation.md`. No sync/archive command was run.
