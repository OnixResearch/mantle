## Context

Existing release verification specs define bounded proof classes such as `self-rebuild-match`, and recent deterministic-proof work added sandbox profile and isolation-evidence concepts. This change pins the implementation-facing contract for a proof unit: run the selected artifact twice from fresh proof stores, compare BLAKE3 digest sets, and emit a receipt that release verification can validate without reading logs.

## Goals / Non-Goals

**Goals:**
- Select exactly one proof unit target per receipt: Mantle self-build/release artifact by default, or a smaller release artifact when chosen explicitly.
- Record exact proof inputs: workflow/version, source tree BLAKE3, vendor/input bundle BLAKE3, selected provider kind, toolchain/stage roots, logical store prefix, sandbox profile identity, and artifact output selection.
- Run at least two clean rebuilds in distinct fresh proof stores/output roots.
- Fail closed for missing/unsupported sandbox evidence, digest drift, provider-kind mismatch, unsupported workflow version, impure/practical mode, malformed receipt fields, or store/output reuse.
- Keep the human and JSON claim bounded to the selected artifact and recorded environment.

**Non-Goals:**
- No full-source bootstrap or whole-system determinism claim.
- No new witness/social quorum semantics.
- No network-enabled deterministic proof profile unless a future spec defines it.
- No non-BLAKE3 Mantle-owned final proof identity.

## Decisions

### 1. Proof unit before proof result

**Choice:** The orchestrator builds a `DeterministicProofUnit` before running: target artifact identity, workflow/version, provider kind, source/vendor BLAKE3, toolchain/stage roots, sandbox profile, output selection, and proof-store allocation plan.

**Rationale:** If the unit is implicit, later validation cannot distinguish “same artifact rebuilt twice” from “some nearby command happened to match.”

**Implementation:** The unit should be serialized into the receipt and included in the receipt BLAKE3 digest. Provider kind must be a closed value shared with existing provider-linkage evidence.

### 2. Canonical v1 receipt

**Choice:** Define `mantle-deterministic-proof-receipt-v1` as canonical compact JSON with a BLAKE3 digest over canonical bytes excluding the self-digest field.

**Rationale:** A versioned typed artifact lets release verification reject drift or unsupported fields without parsing ad hoc logs.

**Implementation:** Required top-level fields:
- `workflow` and `workflow_version`
- `proof_unit` / target artifact identity
- `selected_provider_kind`
- `source_blake3` and `vendor_blake3`
- `toolchain_stage_roots`
- `sandbox_profile_identity` with supported prefix `mantle-proof-sandbox-v1:`
- `rebuild_a` and `rebuild_b` artifact digest sets, plus optional additional runs
- per-run proof store/output identities and anti-reuse evidence
- closed `verdict`
- `receipt_blake3`

### 3. Fresh per-run roots are proof evidence

**Choice:** Every comparison run gets a distinct proof store and output root, and receipt finalization records their canonical identities plus created-empty/anti-reuse evidence generated before the run.

**Rationale:** Matching output digests are not evidence of determinism if the second run reused the first run’s output.

**Implementation:** Reject paths equal to each other, nested inside each other, equal to the default store, equal to the main release-reproduce store/output, or already containing the derivation-under-test output.

### 4. Sandbox evidence is mandatory

**Choice:** Every proof run must execute under a supported sandbox envelope and record a `mantle-proof-sandbox-v1:*` profile identity. Direct-host, missing, bypassed, or unsupported sandbox evidence is non-promoting.

**Rationale:** A deterministic proof that allows ambient host state is not the bounded claim this receipt is meant to encode.

### 5. `self-rebuild-match` is the strongest claim here

**Choice:** The receipt verdict may be `self-rebuild-match` only when rebuild A and rebuild B have identical canonical BLAKE3 artifact digest sets for the selected proof unit and all identity/sandbox/provider checks pass. All other cases map to closed non-promoting verdicts such as `mismatch`, `missing-evidence`, `reused-store`, `unsupported-workflow`, `unsupported-sandbox`, `provider-kind-mismatch`, or `impure-mode`.

**Rationale:** The intended bounded claim is local repeated clean rebuild agreement, not full bootstrap reproducibility.

## Risks / Trade-offs

- **Long rebuild time** → Keep the first positive fixture small; use the same contract later for the full Mantle self-build.
- **Host bwrap availability** → Unit tests can use fake sandbox runners for receipt/finalizer behavior, with separate gated tests for real sandboxing.
- **Overclaiming** → CLI/docs must phrase the result as: “this artifact rebuilt twice from these recorded inputs under this sandbox and matched.”
