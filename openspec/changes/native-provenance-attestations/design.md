## Context

crunch already has the raw ingredients for a strong attestation story:

- `CrunchDerivation` and `nix_compat::Derivation` capture build intent
- `crunch-project` resolves and locks source, mirror, and patch provenance
- `PathInfo` captures final artifact identity, references, and signatures
- BLAKE3 already anchors store-path and content identity across the system

What crunch does not have is a first-class way to preserve that information as
a native attestation object. Existing SBOM ecosystems are not a good fit here:
they flatten build/runtime edges, blur claims and facts, and force crunch into
package/document identifiers that are weaker than store-path identities.

Terminology for this design:
- **Attestation**: the canonical persisted object and its digest
- **Provenance**: the claims and facts carried by an attestation

## Goals / Non-Goals

**Goals:**
- Define a crunch-native attestation model
- Keep declared claims separate from observed facts
- Make per-artifact, per-closure, and per-project attestations deterministic
- Use BLAKE3 digests as the native identity and verification mechanism
- Integrate attestations into build, substitution, and project workflows
- Allow package authors to declare provenance claims in Nickel without
  changing derivation hashes

**Non-Goals:**
- SPDX/CycloneDX compatibility in v1
- Automatic license or CVE scanning
- Proving real-world truth of user-authored claims
- Making provenance metadata part of the hashed derivation recipe by default

## Decisions

### 1. Use a crunch-native attestation format

**Choice:** crunch owns the attestation schema. It is not a thin wrapper around
SPDX or CycloneDX.

**Rationale:** crunch has richer native structure than generic SBOM formats:
logical store paths, typed build/runtime/source/patch edges, and closure
membership. Preserving that structure gives a better verification story and a
cleaner canonical form.

**Alternative:** adopt SPDX/CycloneDX as the source of truth. Rejected because
it would make the internal model weaker, more ambiguous, and harder to verify.

### 2. Split claims from facts

**Choice:** attestation objects have separate declared claims and observed
facts sections.

**Rationale:** package metadata such as supplier, homepage, or license is a
claim. Store path, output name, runtime references, and content hash are
facts crunch can compute. Mixing them hides which parts are mechanically
verified and which parts are user assertions.

### 3. Make artifact attestation the unit of storage

**Choice:** the primary persistent object is a per-output artifact attestation.
Closure and project attestations are assembled from those artifact objects.

**Rationale:** store outputs are the natural unit of identity, caching, and
substitution. Per-output objects deduplicate well and let closure/project
views compose cleanly.

### 4. Canonicalize first, render later

**Choice:** crunch defines one canonical native representation with one digest.
Human-readable Nickel/JSON renderings are projections of that canonical form.

**Rationale:** deterministic hashing should not depend on pretty printing or
external exporter conventions. Canonicalization must happen before rendering.

### 5. Use BLAKE3 digests for native identity

**Choice:** native attestation identity is BLAKE3-based at four levels:
- declared claims hash
- build-graph hash
- artifact content hash
- final attestation hash

**Rationale:** the system already uses BLAKE3 heavily. Reusing it keeps the
mental model simple and allows Merkle-style closure and project attestations.

### 6. Keep package claims out of derivation hashing by default

**Choice:** builder-layer Nickel accepts optional provenance claims metadata,
but that metadata is not part of the derivation hash unless a future explicit
mode opts into that.

**Rationale:** changing a homepage, supplier, or descriptive field should not
force a rebuild of the world.

### 7. Treat closure and project attestations as Merkle summaries

**Choice:** closure and project attestation digests are computed from sorted
member artifact-attestation digests plus sorted typed edges and explicit root
or project selections.

**Rationale:** this gives deterministic identity, cheap diffing, and simple
verification without rescanning the world.

## Architecture

### Native object model

The attestation layer should model native crunch concepts directly:

- **Source node**: git checkout, tarball, file, local path, patch
- **Recipe node**: derivation/build recipe identity
- **Artifact node**: one output store path
- **Project node**: manifest + lockfile scoped identity
- **Closure node**: rooted transitive view over artifacts and edges

Typed edges include:
- `build-input`
- `runtime-reference`
- `produced-by`
- `fetched-from`
- `patched-by`
- `member-of-closure`
- `declared-by-project`

### Integration points

- **Nickel/builders**: package definitions provide optional provenance claims
- **Conversion/build pipeline**: recipe identity and declared claims are bound
  to the evaluated derivation
- **Store layer**: final output facts and runtime references are attached after
  `PathInfo` exists, and the store retrieves attestations by logical store path
- **Project layer**: manifest, lockfile, mirrors, and patch facts are folded
  into project and source attestations

### Verification boundary

The pure core should normalize claims and facts into canonical attestation
objects and compute digests. Imperative code performs file I/O, store access,
and CLI rendering around that core.

## Risks / Trade-offs

**[Schema lock-in]** A native format is powerful but commits crunch to its own
schema discipline. Mitigation: version the canonical representation and keep
renderers thin.

**[Feature scope]** Attestations can sprawl into policy, scanning, and security
analytics. Mitigation: keep v1 focused on deterministic identity, typed edges,
and verification.

**[Metadata abuse]** Users may put arbitrary prose into claims. Mitigation:
keep claim fields structured and small, with clear contracts in builder-layer
Nickel.

**[Storage growth]** Per-output artifact attestations and closure/project
summaries add state. Mitigation: key them by digest, deduplicate aggressively,
and compose aggregate objects from member digests instead of repeating full
payloads.
