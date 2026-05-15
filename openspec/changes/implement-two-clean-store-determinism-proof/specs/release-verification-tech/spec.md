## MODIFIED Requirements

### Requirement: Release verification consumes two-clean-store proof receipts without overclaiming

Release verification MUST treat deterministic proof receipts as bounded local rebuild evidence only. A receipt may support proof class `self-rebuild-match` only when it validates as `mantle-deterministic-proof-receipt-v1`, records the selected proof unit and exact inputs, records at least rebuild A and rebuild B from distinct clean proof stores, records supported sandbox profile evidence for every run, and the canonical BLAKE3 artifact digest sets for rebuild A and rebuild B match.

A deterministic proof receipt MUST record:

- workflow identity and workflow version;
- selected proof-unit target and output selection;
- selected provider kind;
- source tree BLAKE3 and vendor/input bundle BLAKE3;
- toolchain/stage roots and logical store prefix;
- sandbox profile identity with supported prefix `mantle-proof-sandbox-v1:`;
- rebuild A and rebuild B artifact digest sets using BLAKE3;
- per-run clean-store/output-root identities and anti-reuse evidence;
- closed verdict and receipt BLAKE3.

Release verification MUST fail closed and MUST NOT report `self-rebuild-match` when the receipt has an unsupported workflow version, missing required input digest, non-BLAKE3 final proof identity, mismatched provider kind, missing/unsupported/direct-host sandbox evidence, reused proof store/output root, impure/practical-mode audit evidence, malformed receipt fields, or differing rebuild A/B artifact digest sets.

The human-facing claim MUST remain bounded to the selected artifact and recorded assumptions, using language equivalent to: “this artifact rebuilt twice from these recorded inputs under this sandbox and matched.” It MUST NOT claim full-source bootstrap reproducibility, cross-platform determinism, global Mantle determinism, or policy/social witness sufficiency unless separate evidence proves those broader claims.

#### Scenario: Two clean sandboxed rebuilds match

- GIVEN a deterministic proof receipt for one selected release artifact
- AND the receipt records workflow/version, selected provider kind, source/vendor BLAKE3, toolchain/stage roots, and sandbox profile identity `mantle-proof-sandbox-v1:*`
- AND rebuild A and rebuild B used distinct fresh proof stores/output roots
- AND both rebuild artifact digest sets use canonical BLAKE3 identities
- WHEN release verification validates the receipt
- AND rebuild A and rebuild B artifact digest sets match
- THEN verification may report proof class `self-rebuild-match`
- AND the report scopes the claim to the selected artifact and recorded sandbox/input identities

#### Scenario: Reused proof store blocks self-rebuild match

- GIVEN a deterministic proof receipt whose rebuild A/B artifact digest sets match
- BUT two proof runs share a proof store/output root or one run used the main reproduce output or default store for the proof-unit output
- WHEN release verification evaluates the receipt
- THEN it rejects `self-rebuild-match`
- AND the report identifies reused proof-run storage as the blocker

#### Scenario: Missing sandbox evidence blocks self-rebuild match

- GIVEN a deterministic proof receipt whose rebuild A/B artifact digest sets match
- BUT a proof run lacks sandbox profile evidence or records direct-host/unsupported evidence
- WHEN release verification evaluates the receipt
- THEN it rejects `self-rebuild-match`
- AND the report identifies unsupported deterministic proof sandbox evidence as the blocker

#### Scenario: Provider kind mismatch blocks self-rebuild match

- GIVEN a deterministic proof receipt with selected provider kind `source-root`
- BUT proof linkage, prerequisites, or rebuild-run evidence records a different provider kind
- WHEN release verification validates the receipt
- THEN validation fails closed
- AND the report identifies provider kind mismatch as the blocker

#### Scenario: Unsupported workflow version blocks self-rebuild match

- GIVEN a deterministic proof receipt with an unknown workflow version
- WHEN release verification validates the receipt
- THEN validation fails closed
- AND the receipt cannot contribute to `self-rebuild-match`

#### Scenario: Digest mismatch blocks self-rebuild match

- GIVEN rebuild A and rebuild B completed under supported sandbox evidence
- BUT a selected artifact output has different BLAKE3 digests between the runs
- WHEN release verification finalizes the receipt
- THEN the verdict is a non-promoting mismatch
- AND Mantle MUST NOT claim `self-rebuild-match`
