## Why

Mantle/Crunch release determinism must be a machine-checkable proof unit, not a narrative assembled from logs. The highest-ROI next slice is a deterministic proof receipt plus two-clean-store orchestration and anti-reuse guard for one bounded artifact target (initially the Mantle self-build/release artifact or a smaller release artifact selected by implementation).

The proof must say only: “this artifact rebuilt twice from these recorded inputs under this sandbox and matched.” It must not imply full bootstrap reproducibility or global determinism while bootstrap parity blockers remain open.

## What Changes

- Define a deterministic proof unit with exact input identities: source tree BLAKE3, vendor/input bundle BLAKE3, selected bootstrap provider kind, toolchain/stage roots, sandbox profile, workflow identity/version, and selected artifact outputs.
- Orchestrate two or more clean proof rebuilds using fresh stores/output roots and reject reuse of the default store, main reproduce output, or a prior proof run’s store/output.
- Emit a canonical deterministic proof receipt whose final proof identities and artifact digest sets use BLAKE3, not SHA.
- Require supported sandbox evidence for every run with `mantle-proof-sandbox-v1:*` profile identities; fail closed on missing, direct-host, or unsupported sandbox evidence.
- Validate provider-kind linkage so the selected provider kind in the receipt matches the recorded proof inputs and rebuild runs.
- Promote only to `self-rebuild-match` when clean rebuild A and B artifact digest sets match the recorded inputs under the recorded sandbox envelope.

## Capabilities

### New Capabilities
- `build-pipeline.determinism.proof-unit`: bounded proof-unit definition for one selected artifact target.
- `build-pipeline.determinism.two-clean-store-receipt`: canonical two-clean-store receipt schema, orchestration, and anti-reuse validation.

### Modified Capabilities
- `release-verification-tech.reproducibility.claims`: release verification consumes deterministic receipts and refuses overbroad claims.

## Impact

- **Files**: deterministic proof model/schema, release reproduce/self-build orchestration, sandbox proof runner, release verifier, CLI JSON/human output, docs, tests.
- **APIs/CLI**: likely extends `mantle release reproduce --deterministic-proof-runs <N>` / deterministic proof output paths without requiring a new top-level command.
- **Testing**: positive two-clean-store match fixture; negative tests for mismatched digest, missing sandbox evidence, unsupported workflow version, provider kind mismatch, reused store/output, and malformed receipt fields.
- **Non-goals**: full bootstrap parity, social witness policy, global Mantle determinism, accepting impure/practical-mode proof evidence, or claiming broader reproducibility than the selected proof unit.
