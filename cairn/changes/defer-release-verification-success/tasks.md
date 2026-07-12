## Phase 1: Final decision core

- [ ] [serial] Define normalized release verification contributor results, selected policy requirements, and a pure final decision with closed disposition and ordered diagnostics. r[mantle.release_provenance.verification_decision.complete] r[mantle.release_provenance.verification_decision.boundary]
- [ ] [serial] Make contributor coverage exhaustive and add a focused completeness check for newly required evidence policies. r[mantle.release_provenance.verification_decision.completeness] r[mantle.release_provenance.verification_decision.completeness.test]
- [ ] [serial] Add plain pure-core assertions for optional absence, full acceptance, and one or multiple required-policy blockers. r[mantle.release_provenance.verification_decision.boundary.test]

## Phase 2: CLI ordering and rendering

- [ ] [serial] Refactor `cmd_release_verify` to collect normalized facts, aggregate the complete decision, and only then invoke a renderer or select process status. r[mantle.release_provenance.verification_decision.complete]
- [ ] [serial] Render the human success marker only for an accepted final decision and use explicit non-success wording for rejection. r[mantle.operator_diagnostics.release_verification.terminal_verdict]
- [ ] [serial] Version JSON output with top-level `valid`, closed `disposition`, ordered `checks`, and ordered `diagnostics` for acceptance and policy rejection. r[mantle.operator_diagnostics.release_verification.json_contract]
- [ ] [serial] Keep rendering effect-free over the immutable decision and reserve JSON stdout for exactly one payload. r[mantle.operator_diagnostics.release_verification.render_boundary] r[mantle.operator_diagnostics.release_verification.render_boundary.test]

## Phase 3: Positive and negative fixtures

- [ ] [parallel] Add human and JSON success fixtures proving the success marker appears only after every selected check passes. r[mantle.release_provenance.verification_decision.fixtures.positive] r[mantle.operator_diagnostics.release_verification.fixtures.positive]
- [ ] [parallel] Add independent required-reproducibility, deterministic-proof, provider-proof, stack-provenance, external-role, and StageX rejection fixtures. r[mantle.release_provenance.verification_decision.fixtures.negative]
- [ ] [parallel] Assert every human rejection exits nonzero without `release evidence verified` or equivalent success wording. r[mantle.operator_diagnostics.release_verification.fixtures.negative]
- [ ] [parallel] Assert every JSON policy rejection emits one parseable `valid: false` result, ordered diagnostics, and a nonzero exit. r[mantle.operator_diagnostics.release_verification.json_negative]

## Phase 4: Validation and documentation

- [ ] [serial] Document the final validity/disposition contract and migration rule for JSON consumers. r[mantle.operator_diagnostics.release_verification.json_contract]
- [ ] [serial] Run focused release command unit tests and release CLI positive/negative output tests. r[mantle.release_provenance.verification_decision.fixtures.positive] r[mantle.release_provenance.verification_decision.fixtures.negative]
- [ ] [serial] Run Cairn validation and proposal, design, and tasks gates before sync/archive. r[mantle.release_provenance.verification_decision.complete]
