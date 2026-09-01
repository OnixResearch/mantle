## ADDED Requirements

### Requirement: Full bootstrap parity promotion is evidence-derived

r[bootstrap_inventory.full_bootstrap_parity_promotion] Mantle MUST emit a completed full-bootstrap parity claim only when independent current evidence validates every required native-toolchain row, the complete StageX lineage provider, and a source-built Mantle `mantle-deterministic-proof-receipt-v2` fixed point under one consistent source, provider, closure, authority, root action trust, and observed execution identity.

#### Scenario: each parity domain validates independently

GIVEN row-specific native receipts, a StageX lineage receipt, and a Mantle fixed-point proof bundle
WHEN Mantle computes parity
THEN each domain MUST validate its own schema, source/predecessor/output BLAKE3 links, freshness, positive/rejection evidence, fallback status, and claim boundaries before contributing completion
AND repeated digest text, normalized-provider metadata, later-stage success, or another domain's status MUST NOT substitute for missing evidence.

#### Scenario: StageX and self-build evidence agree

GIVEN native parity rows are complete
WHEN StageX and Mantle self-build evidence is evaluated
THEN the lineage and v2 fixed-point receipts MUST agree on audited seed, source state, lineage graph, selected provider, native/Rust closure, protected execution, authority plan, effect policy, and stage inputs
AND scaffold lineage, v1 proof, imported provider authority, fallback events, unapproved reads/effects, mixed identities, or stage digest mismatch MUST keep the relevant axes incomplete.

#### Scenario: root action trust evidence is complete

GIVEN native parity rows and the StageX lineage are complete
WHEN Mantle evaluates the source-built fixed-point domain
THEN the bound root action trust plan and observed execution reconciliation MUST cover every reachable action, fixed or producer-linked executable authority, input authority, output, local-only execution rule, and event-count bound
AND an incomplete adapter, generated-path classification without producer identity, unknown or missing execution event, digest or producer drift, remote execution, cache-only completion, or count-bound violation MUST keep promotion incomplete.

#### Scenario: independent verifier accepts the bundle

GIVEN Mantle exports a promoted bootstrap evidence bundle
WHEN the standalone verifier reads it outside the producer output tree
THEN the verifier MUST recompute BLAKE3 relationships, action-trust counts, planned-versus-observed execution coverage, every required schema, and every cross-receipt edge; reject target-authority and absolute-path dependence; and reproduce the promoted bounded status
AND producer status fields alone MUST NOT authorize acceptance.

#### Scenario: bootstrap promotion does not require witness quorum

GIVEN every bootstrap axis and the independent bundle verifier pass with no build-witness sidecars
WHEN Mantle emits promoted bootstrap and release evidence
THEN bootstrap promotion MUST succeed without selecting or satisfying a witness-quorum policy
AND any later optional-witness or explicit quorum result MUST remain separate and MUST NOT change bootstrap-axis status.

#### Scenario: parity require mode fails closed

GIVEN an operator requires live-bootstrap, Guix, StageX, or all defined axes
WHEN any required row or evidence edge is missing, partial, stale, malformed, inconsistent, or unverified
THEN the CLI MUST exit nonzero with deterministic row/domain diagnostics and machine-readable blocker details
AND it MUST NOT emit the bounded full-bootstrap success claim.

#### Scenario: promoted claim remains bounded

GIVEN all required axes and the independent verifier pass
WHEN human, JSON, release, or operator documentation reports full bootstrap
THEN the claim MUST define its scope as the recorded StageX-seed-to-source-built-Mantle fixed point and identify sources, providers, closure, authority, root action trust, planned and observed execution counts, platform, stages, audits, and digests
AND it MUST explicitly disclaim compiler correctness, seed correctness, kernel isolation, independent rebuild agreement, bit-for-bit release reproducibility, deployment success, and full Cargo compatibility.