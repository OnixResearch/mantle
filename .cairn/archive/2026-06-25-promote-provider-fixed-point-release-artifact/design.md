## Context

`ReleaseEvidenceManifest` already validates that the full self-hosting proof's `proof_linkage.stage2_binary_digest_blake3` appears in `manifest.binaries`. Separately, `ProviderFixedPointProofVerification` validates provider-backed Cargo-free fixed-point proof bundles and reports the fixed-point `stage_binary_digest_blake3`.

Those two evidence paths are currently independent. A release bundle can carry a valid provider proof whose stage binary digest differs from the packaged release binary, and `--require-provider-fixed-point-proof` still treats the provider proof as valid release-adjacent evidence.

## Decisions

### 1. Add a pure release-core binding check

**Choice:** Add a no-I/O core function that takes bundled binary artifacts plus a provider stage binary digest and returns the matched release artifact or a validation error.

**Rationale:** The binding rule is deterministic business logic over known artifact records. Keeping it in `crunch-release-core` lets create, verify, and tests share one rule without filesystem or process dependencies.

### 2. Enforce binding at both create and verify

**Choice:** `release create` rejects mismatched provider proof before writing trusted manifest evidence, and `release verify` marks provider proof verification invalid when the supplied bundled or external proof does not match the release binary set.

**Rationale:** Create-time enforcement prevents new inconsistent bundles. Verify-time enforcement protects older or hand-edited bundles and explicit external proof overrides.

### 3. Report the matched release artifact explicitly

**Choice:** Extend provider fixed-point verification output with optional `release_artifact_relative_path` and `release_artifact_digest_blake3` fields when the binding succeeds.

**Rationale:** Operators need durable JSON/human evidence showing which release artifact was provider-backed, not just that a provider proof was valid in isolation.

## Risks / Trade-offs

- Existing historical bundles with mismatched provider proof sidecars will no longer satisfy `--require-provider-fixed-point-proof`. This is intentional; they remain inspectable as packaged-integrity evidence but cannot claim provider-backed release artifact binding.
- Multi-binary bundles are accepted when any packaged binary matches the provider fixed-point digest. The matched relative path is reported so callers can decide whether that is the intended primary artifact.
- This does not promote deterministic-release or full-bootstrap claims from provider proof evidence alone; it only binds the provider proof to the release artifact set.
