# Design: Bind source-review evidence to releases

## Context

Mantle release manifests already identify deterministic source archives with BLAKE3. Cairn can validate semantic-review receipts, and its workflow-collaboration design can bind signed verifier decisions to an exact claim root. Valence can preserve evidence identity, roles, and non-claims across repository boundaries.

These systems do not yet define a stable Mantle release attachment. Mantle must not trust a producer status string, reviewer display name, or detached signature without exact subject, policy, key, and source linkage.

## Decisions

### Decision: Consume external review decisions instead of owning review workflow

**Choice:** Mantle consumes a stable Cairn-produced and Valence-identified source-review discharge artifact. Mantle does not create reviewer decisions, run review agents, manage collaboration state, or interpret findings.

**Rationale:** Cairn owns lifecycle decisions. Valence owns evidence identity and linkage. Mantle owns release source and artifact identity.

### Decision: Bind approval to the exact released source

**Choice:** The attachment binds the release source archive BLAKE3, source revision or tree identity when supplied, Cairn claim root, review policy digest, decision, reviewer full-key BLAKE3 identities, canonical signed-statement identities, Valence evidence identity, and required non-claims.

The verifier recomputes the source archive digest from release evidence and rejects mismatched or stale review subjects.

**Rationale:** Approval for one source tree must not authorize another tree, regenerated archive, or later revision.

### Decision: Keep review policy optional for generic releases

**Choice:** Generic release verification reports source review as `not-required` when no reviewed-source policy is selected. An explicit reviewed-source policy defines the required distinct approval count, accepted reviewer keys or roles, author-exclusion rule, and currentness inputs.

The StageX-inspired preset uses a named two-reviewer minimum. The numeric value lives in typed policy, not as an unexplained literal in verifier logic.

**Rationale:** The mechanism can ship before Mantle makes it a universal release gate. Operators can adopt stronger policy deliberately.

### Decision: Keep reviewer, releaser, and builder-witness roles separate

**Choice:** Source-review approvals, release signatures, and build-witness attestations use different domain-separated statements and policy roles. No signature or identity counts across roles unless a future explicit policy evaluates each role separately.

Build-witness quorum remains optional. A reviewed-source profile does not imply witness quorum.

**Rationale:** A reviewer approves source under review policy. A witness reports rebuilt output facts. These are not interchangeable claims.

### Decision: Verify cryptography and policy independently

**Choice:** Mantle recomputes canonical review statement bytes and uses the stack-selected independent Ed25519 verifier. It deduplicates approvals by full public-key BLAKE3 identity, applies revocation and required-role policy, and rejects producer-only `approved` fields.

**Rationale:** A valid external envelope or matching display name does not prove approval authority.

### Decision: Fail closed only for selected policy

**Choice:** If reviewed-source policy is required, missing, stale, malformed, tampered, revoked, role-confused, or threshold-insufficient review evidence blocks that policy. If review is optional, Mantle preserves valid evidence and deterministic diagnostics without promoting it or blocking an unrelated generic release solely because review evidence is absent.

**Rationale:** Optional evidence and required admission must remain distinct.

### Decision: Preserve the functional core and imperative shell

**Choice:** Pure core functions validate source-review DTOs, identity links, signature observations, role separation, reviewer counts, and policy outcomes. The shell reads files, invokes external artifact loading, resolves release paths, and renders output.

**Rationale:** Source-review admission logic must be testable without filesystem, process, clock, network, or environment state.

## External Contract Gate

Implementation must stop with a durable blocker if Cairn and Valence do not provide stable, versioned producer and identity contracts. Placeholder schemas, copied status fields, or hand-authored fake approvals cannot satisfy this change.

## Risks / Trade-offs

- Cross-repository schema coordination can delay implementation.
- Reviewer-key rotation and revocation freshness remain operator-policy inputs.
- A two-reviewer threshold can still represent one organization or correlated process.
- Signed approval proves attributable policy action, not source correctness or reviewer competence.
- Adding release evidence members changes release bundle and machine-contract surfaces.

## Claim Boundary

A satisfied reviewed-source policy proves that the configured distinct authorized reviewer keys signed approval statements bound to the exact released source and policy. It does not prove source correctness, review quality, build correctness, reproducibility, witness independence, or deployment safety.
