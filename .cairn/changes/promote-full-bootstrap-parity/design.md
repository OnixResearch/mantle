## Context

The current parity report correctly keeps every axis incomplete. It also exposes the exact remaining evidence seams: partial native rows, scaffold-only StageX lineage, and a historical self-build receipt using deterministic proof v1. Once predecessor changes produce current evidence, promotion must remain a deterministic read/validation operation rather than a status override.

## Decisions

### Decision: promotion consumes independent evidence domains

**Choice:** Evaluate early native rows, final native rows, StageX lineage, and Mantle fixed-point proof independently, then compute axis completion from their conjunction. One receipt cannot satisfy another domain by repeating its digest fields.

**Rationale:** Independent domains prevent a successful final provider or self-build from hiding incomplete lineage.

### Decision: preserve fail-closed compatibility surfaces

**Choice:** Keep established JSON schema and row IDs, including compatibility identifiers, unless a versioned migration is necessary. Project-facing prose and titles use Mantle. Missing legacy consumers are not a reason to accept stale evidence.

**Rationale:** Promotion should not combine a trust change with gratuitous API churn.

### Decision: require root action trust evidence from the fixed-point domain

**Choice:** Promotion requires the complete fixed-point action trust plan, observed execution reconciliation, and their v2 receipt links. It rejects incomplete adapters, path-only generated authority, producerless executables, unknown or missing events, digest or producer drift, count-bound violations, remote execution, and cache-only completion. The report complements the StageX seccomp audit and cannot replace it.

**Rationale:** StageX has detailed protected execution evidence, while the six-stage fixed-point summary spans the full proof. Promotion needs an explicit link between these levels before it can emit a full-bootstrap claim.

### Decision: add a second implementation for bundle verification

**Choice:** A standalone Rust checker reads the exported promoted bundle, recomputes BLAKE3 bindings, verifies cross-receipt identities and status, recomputes action-trust counts and planned-versus-observed coverage, and rejects path-dependent or target-authority evidence. Positive and negative fixtures cover each edge.

**Rationale:** The producer and parity collector sharing one code path would leave correlated serialization/validation defects unchecked.

### Decision: derive claims from completed axes

**Choice:** Human and JSON reports name the exact completed axes, evidence, planned action count, observed event count, unmatched count, and action-trust blockers. “Full bootstrap” is emitted only for the defined StageX-to-Mantle fixed-point scope. Compiler correctness, seed correctness, kernel isolation, independent rebuild agreement, release reproducibility, deployment, and full Cargo compatibility remain non-claims.

**Rationale:** A strong bounded claim is more useful than an ambiguous universal one.

### Decision: keep build-witness policy separate from bootstrap promotion

**Choice:** A complete full-bootstrap bundle can pass parity promotion with no build-witness sidecars and no witness-quorum policy. Promoted release evidence may feed later `optional-witness` or explicit `witness-quorum` evaluation, but that separate result cannot change bootstrap-axis status.

**Rationale:** Full bootstrap and external rebuild agreement prove different facts. Quorum remains an operator-selected social-policy option, not a hidden bootstrap requirement.

### Decision: require current committed evidence

**Choice:** Promotion uses evidence produced from the implementation commit or a committed ancestor explicitly named by the proof source descriptor. Scaffold, synthetic, stale-schema, target-only, or uncommitted-source evidence fails closed.

**Rationale:** Historical success cannot validate changed implementation or policy.

## Risks / Trade-offs

- Independent verification can expose producer/checker disagreement late; that disagreement blocks promotion.
- Complete action lists can be large. The promoted schema must retain deterministic order and explicit count limits.
- A missing native-provider or Rust-unit adapter blocks promotion instead of producing a partial report.
- Existing external consumers may rely on compatibility row names, so versioning must be deliberate.
- The final report remains scoped and will not imply formal correctness.