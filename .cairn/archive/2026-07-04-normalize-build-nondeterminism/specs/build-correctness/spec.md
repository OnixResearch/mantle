## ADDED Requirements

### Requirement: Determinism normalization policy

r[build_correctness.determinism_normalization_policy] Mantle MUST bind a strict determinism normalization policy for proof-grade builds, covering time, timezone, locale, umask, temp roots, host/user metadata, modeled randomness, and order-sensitive output processing.

#### Scenario: equivalent builds converge

GIVEN two strict builds use equivalent declared inputs and the same determinism normalization policy
AND ambient host time, locale, umask, temp roots, user names, and environment noise differ
WHEN Mantle admits build evidence for those outputs
THEN the admitted output digests and relevant receipt fields MUST match or the divergence MUST be reported deterministically
AND the receipt MUST bind the normalization policy used for the comparison.

#### Scenario: unsupported normalization blocks strong claims

GIVEN a requested strict build or proof requires a normalization control that the current executor cannot enforce
WHEN Mantle evaluates strong build-correctness, self-hosting, release, or reproducibility eligibility
THEN Mantle MUST mark the affected claim as blocked or unsupported
AND it MUST NOT silently continue with a partial normalization policy.

#### Scenario: nondeterministic divergence is diagnostic evidence only

GIVEN two proof-grade runs from equivalent declared inputs produce different content-addressed output digests
WHEN Mantle summarizes the proof result
THEN the report MUST identify the first modeled divergent surface when available
AND it MUST NOT claim deterministic release, global reproducibility, or strong cross-run equivalence for the divergent artifact.
