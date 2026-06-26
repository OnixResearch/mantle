## Context

The release artifact binding change makes provider fixed-point proof evidence invalid for release verification unless the proof's fixed-point stage binary digest appears in the release manifest binary set. To create evidence that satisfies both the existing self-hosting proof linkage and the new provider binding, the release bundle carries two binaries:

1. the provider fixed-point stage2 `mantle` binary, matched by provider proof evidence;
2. the legacy full self-hosting proof stage2 binary, matched by the existing full proof linkage.

The deterministic proof helper copies both binary artifacts into each clean proof output so deterministic release eligibility covers the complete manifest binary set.

## Decisions

### 1. Keep generated proof payloads ignored

**Choice:** Leave the release bundle, deterministic proof receipt, verification JSON, and portable replay scratch under `target/` and record only concise lifecycle evidence in Cairn.

**Rationale:** Generated release payloads are large and already content-addressed by BLAKE3 fields in the verifier output. The durable claim needs transcript and digest evidence, not multi-GiB payloads.

### 2. Require both deterministic and provider gates in replay

**Choice:** The positive replay runs `mantle release verify --require-deterministic-release --require-provider-fixed-point-proof` against copied bundle/proof artifacts using a copied current verifier binary. The negative replay withholds required deterministic proof material.

**Rationale:** This proves the portable verifier consumes copied bundle/proof sidecars and fails closed when required evidence is absent. The packaged provider proof binary predates the provider-bound verifier flag, so the evidence records that limitation and makes regenerating a current-code provider fixed point the next milestone.

### 3. Preserve bounded non-claims

**Choice:** Evidence states that provider-bound deterministic release evidence does not prove full bootstrap reproducibility, compiler correctness, deploy success, or full Cargo compatibility.

**Rationale:** The proof binds specific packaged artifacts and recorded proof inputs only.

## Risks / Trade-offs

- The bundle contains two binaries because current release evidence still requires the full self-hosting proof stage2 digest to appear in the release binary set. The provider-bound artifact is explicitly `binaries/01-mantle` in verifier output.
- The proof is release-artifact evidence, not a replacement for StageX lineage or minimized bootstrap-root evidence.
