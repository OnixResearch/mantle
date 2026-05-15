## Phase 1: Proof unit and receipt contract

- [ ] [serial] Define the deterministic proof unit model with target artifact identity, workflow/version, selected provider kind, source/vendor BLAKE3, toolchain/stage roots, sandbox profile identity, and selected outputs.
- [ ] [serial] Define the canonical `mantle-deterministic-proof-receipt-v1` schema and BLAKE3 self-digest helper.
- [ ] [parallel] Add validator tests for missing required fields, unsupported workflow version, non-BLAKE3 final proof identities, malformed per-run identities, and provider kind mismatch.
- [ ] [parallel] Add canonical-order tests proving receipt bytes and receipt BLAKE3 are stable across insertion/discovery order.

## Phase 2: Two-clean-store orchestration

- [ ] [serial] Implement proof-run planning that allocates at least rebuild A and rebuild B with distinct fresh proof stores and output roots.
- [ ] [parallel] Add anti-reuse validation rejecting the default store, main release-reproduce output/store roots, duplicate or nested per-run roots, and pre-existing derivation-under-test outputs.
- [ ] [parallel] Record clean-store identity, output-root identity, sandbox profile identity, hermeticity mode, and ambient perturbation case for every proof run.
- [ ] [depends:Phase 2] Ensure proof rebuild commands receive the run-specific store/output roots and cannot satisfy the selected proof unit from a prior run.

## Phase 3: Sandbox and receipt finalization

- [ ] [serial] Require supported `mantle-proof-sandbox-v1:*` sandbox evidence for each proof run and fail closed on direct-host, missing, bypassed, or unsupported sandbox evidence.
- [ ] [depends:Phase 2] Finalize receipts by comparing rebuild A/B canonical per-output BLAKE3 digest sets and mapping all outcomes to closed verdict values.
- [ ] [parallel] Wire release verification to accept `self-rebuild-match` only for validated v1 receipts with matching BLAKE3 digest sets, supported sandbox evidence, matching provider kind, and no anti-reuse blockers.
- [ ] [parallel] Add negative tests for mismatched digest, missing sandbox evidence, unsupported workflow version, provider kind mismatch, impure/practical mode, reused proof store/output, and malformed receipt fields.

## Phase 4: Operator surface and documentation

- [ ] [depends:Phase 3] Update CLI JSON/human output to report receipt path, receipt BLAKE3 digest, proof unit, per-run root identities, sandbox profile identity, verdict, and blockers.
- [ ] [parallel] Document the bounded claim text: “this artifact rebuilt twice from these recorded inputs under this sandbox and matched,” and distinguish it from full-source bootstrap/global determinism.
- [ ] [serial] Run focused unit/integration tests and `openspec validate implement-two-clean-store-determinism-proof --strict` before implementation completion.
