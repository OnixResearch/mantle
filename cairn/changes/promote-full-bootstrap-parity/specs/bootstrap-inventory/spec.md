## ADDED Requirements

### Requirement: Full bootstrap parity promotion is evidence-derived

r[bootstrap_inventory.full_bootstrap_parity_promotion] Mantle MUST emit a completed full-bootstrap parity claim only when independent current evidence validates every required native-toolchain row, the complete StageX lineage provider, and a source-built Mantle `mantle-deterministic-proof-receipt-v2` fixed point under one consistent source, provider, closure, and authority identity.

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

#### Scenario: independent verifier accepts the bundle

GIVEN Mantle exports a promoted bootstrap evidence bundle
WHEN the standalone verifier reads it outside the producer output tree
THEN the verifier MUST recompute BLAKE3 relationships, validate every required schema and cross-receipt edge, reject target-authority and absolute-path dependence, and reproduce the promoted bounded status
AND producer status fields alone MUST NOT authorize acceptance.

#### Scenario: parity require mode fails closed

GIVEN an operator requires live-bootstrap, Guix, StageX, or all defined axes
WHEN any required row or evidence edge is missing, partial, stale, malformed, inconsistent, or unverified
THEN the CLI MUST exit nonzero with deterministic row/domain diagnostics and machine-readable blocker details
AND it MUST NOT emit the bounded full-bootstrap success claim.

#### Scenario: promoted claim remains bounded

GIVEN all required axes and the independent verifier pass
WHEN human, JSON, release, or operator documentation reports full bootstrap
THEN the claim MUST define its scope as the recorded StageX-seed-to-source-built-Mantle fixed point and identify sources, providers, closure, authority, platform, stages, audits, and digests
AND it MUST explicitly disclaim compiler correctness, seed correctness, kernel isolation, independent rebuild agreement, bit-for-bit release reproducibility, deployment success, and full Cargo compatibility.