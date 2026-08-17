## MODIFIED Requirements

### Requirement: Global reproducibility release surface evidence

r[verification_evidence.global_reproducibility_release_surface_evidence] Mantle MUST provide deterministic release-bundle-derived surface evidence for global reproducibility reports without weakening the global admission gate.

#### Scenario: release evidence helper derives accepted stage2 surface evidence

GIVEN a release bundle, verification directory, final release verification JSON, digest-bound global universe, and policy name a stage2 self-hosting artifact
WHEN Mantle derives global reproducibility surface evidence from those inputs
THEN the evidence MUST bind the universe digest, policy digest, action receipt digest, source acquisition digest, toolchain/proof digest, strict hermeticity digest, output digest set, and policy-counted witness identity for the named surface
AND the global reproducibility evaluator MUST be able to admit that surface only when the generated evidence satisfies the existing policy.

#### Scenario: verified provider fixed-point artifacts can satisfy strict/fresh surface evidence

GIVEN a release universe includes a provider fixed-point artifact and the release bundle contains provider proof material
WHEN Mantle derives surface evidence for that artifact
THEN it MUST validate the provider proof bundle before setting strict hermeticity or fresh rebuild-store evidence for that surface
AND it MUST bind the provider surface to the proof metadata digest, source-built toolchain closure policy digest, verifier stage binary digest, release artifact digest, and policy-counted witness identity.

#### Scenario: invalid provider fixed-point artifacts fail closed

GIVEN a release universe includes a provider fixed-point artifact whose provider proof is missing, invalid, weak, copied incompletely, or digest-mismatched against the release artifact
WHEN Mantle derives surface evidence for that artifact
THEN the evidence MUST include an unsupported-surface reason or equivalent blocker-producing fact naming the concrete verifier blocker
AND the final global report MUST remain blocked for that surface instead of promoting the full release universe.

#### Scenario: helper stays separate from final global admission

GIVEN release-derived surface evidence has been written
WHEN an operator wants to claim global reproducibility
THEN the operator MUST still run the global reproducibility evaluator to produce a `mantle-global-reproducibility-report-v1`
AND helper output alone MUST NOT be described as an eligible global report.
