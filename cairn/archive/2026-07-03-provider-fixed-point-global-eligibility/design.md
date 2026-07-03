## Context

`mantle release global-reproducibility-evidence` derives surface evidence from release bundle facts and lets the global evaluator make the final admission decision. Its first version intentionally blocked provider fixed-point handoff artifacts because the helper did not inspect the strict/fresh provider proof verifier output.

The provider fixed-point verifier already validates the evidence we need for this artifact class: fixed-point schema/status, source-built toolchain closure claim, zero seed exceptions, validated Rust source provider, matching preflight closure policy digest, stage success, cargo marker absence, successful smoke status, zero failed units, matching stage binary digests, matching closure policy digests, and successful stage receipts.

## Decisions

### 1. Reuse the provider verifier instead of duplicating proof logic

**Choice:** The release helper converts `verify_provider_fixed_point_proof_bundle(...)` into small global-proof facts and admits the provider surface only when `valid=true`, the verifier returns meta and closure-policy digests, and the verifier's stage binary digest equals the release artifact digest.

**Rationale:** The verifier is the authoritative checker for provider fixed-point proof correctness. Reusing it avoids a second, weaker interpretation of proof metadata while preserving the existing global evaluator as the final policy gate.

### 2. Strict/fresh evidence is evidence-derived, not assumed from artifact class

**Choice:** Provider fixed-point surfaces set `strict_hermeticity=true` and `fresh_rebuild_store=true` only after the verifier succeeds. The surface binds toolchain provenance to the source-built closure policy digest and hermeticity evidence to the proof metadata digest. Missing or invalid verifier facts keep strict/fresh false and attach a deterministic unsupported reason.

**Rationale:** This turns the former coarse artifact-class blocker into explicit evidence checks. It also keeps a concrete blocker when the release bundle lacks strict/fresh proof material.

### 3. Copied proof bundles should validate from copied contents

**Choice:** When provider proof metadata records absolute paths under the original `bundle_dir`, verifier path resolution prefers matching files under the current proof directory if they exist.

**Rationale:** Release evidence bundles include copied stage binaries and receipts. Validation should prove copied bundle sufficiency instead of accidentally depending on the publisher's original scratch path.

## Risks / Trade-offs

- The provider fixed-point proof still carries bounded non-claims for full Cargo compatibility, compiler correctness, and Crunch/Nix bootstrap replacement. The global eligibility claim remains limited to the declared release universe and policy.
- The helper still consumes a final release-verify JSON produced elsewhere; operators should run current release verification before deriving global evidence.
- Future provider proof schema changes must update the verifier first, then the helper's small fact adapter.
