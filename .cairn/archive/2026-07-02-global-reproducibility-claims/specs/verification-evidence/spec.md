## ADDED Requirements

### Requirement: Global reproducibility claim admission

r[verification_evidence.global_reproducibility_claim_admission] Mantle MUST NOT claim global reproducibility unless a durable global reproducibility report proves every build surface in an explicit, digest-bound universe satisfied the configured reproducibility policy.

#### Scenario: global universe is explicit and digest-bound

GIVEN an operator asks Mantle to evaluate or report global reproducibility
WHEN the global reproducibility report is produced
THEN the report MUST bind a universe manifest digest, policy digest, report schema version, included build surfaces, target systems, source acquisition modes, toolchain routes, cache/substitution modes, and release artifact sets
AND every excluded or unsupported surface MUST be listed as an explicit non-claim or blocker.

#### Scenario: scoped release evidence does not imply global reproducibility

GIVEN a release has deterministic-release evidence, provider fixed-point evidence, or external independent witness agreement
WHEN Mantle summarizes the release or broader project status
THEN the summary MAY claim only the release-scoped class proven by that evidence
AND it MUST NOT call Mantle globally reproducible unless the global reproducibility report for the declared universe is eligible.

#### Scenario: every included surface has receipt coverage

GIVEN a build surface is included in the global reproducibility universe
WHEN Mantle evaluates global reproducibility
THEN the report MUST bind action-correct receipts, source acquisition evidence, toolchain provenance, strict hermeticity evidence, output digest evidence, and independent replay evidence for that surface
AND missing, stale, mismatched, weak-hermeticity, reused-store, unsupported, or policy-incomplete evidence MUST block the global claim.

#### Scenario: independent replay matrix is policy-satisfied

GIVEN the configured reproducibility policy requires independent replay across operator domains, host classes, or ambient perturbation axes
WHEN Mantle evaluates the witness matrix
THEN each included surface MUST satisfy the policy-bound quorum with matching BLAKE3 output digest sets
AND skipped, unknown-key, revoked, same-domain, digest-mismatched, or policy-insufficient witnesses MUST be counted separately from accepted witnesses.

#### Scenario: global claim fails closed

GIVEN any included surface lacks required evidence or produces mismatched output digests
WHEN the global reproducibility report is rendered
THEN the report MUST set the claim class to a blocked or non-global value
AND the report MUST preserve deterministic blocker details sufficient to identify the failing surface, evidence class, expected digest, observed digest when present, and required next action.

### Requirement: Global reproducibility reports

r[verification_evidence.global_reproducibility_reports] Mantle MUST emit deterministic, reviewable global reproducibility reports before any operator-facing global reproducibility claim can be made.

#### Scenario: report has stable identity

GIVEN Mantle evaluates a global reproducibility universe and policy
WHEN it emits a report
THEN the report MUST use schema `mantle-global-reproducibility-report-v1`, deterministic ordering, BLAKE3 report digesting, and stable claim classes
AND the report MUST include the universe digest, policy digest, generated evidence digests, accepted witness identities, blocker list, non-claim list, and final claim class.

#### Scenario: eligible report names exact claim boundary

GIVEN every included surface satisfies the configured policy
WHEN Mantle renders human or JSON output
THEN the output MUST name the exact universe digest, policy digest, included surface count, witness policy, and final claim class
AND it MUST state that the claim applies only to that universe and policy, not to future code, undeclared frontends, undeclared target systems, compiler correctness, deploy success, or physical-target determinism.

#### Scenario: blocked report remains useful evidence

GIVEN one or more included surfaces fail the configured policy
WHEN Mantle renders human or JSON output
THEN the output MUST remain deterministic and include all discovered accepted evidence plus blockers
AND it MUST NOT discard successful scoped evidence merely because the global claim failed.

#### Scenario: stale report cannot be promoted

GIVEN a global reproducibility report was produced for an older source tree, policy, universe, release bundle, toolchain closure, or witness set
WHEN Mantle reports current project or release status
THEN it MUST reject or label the report as stale unless current digests match the report-bound inputs
AND it MUST NOT use stale report evidence to claim current global reproducibility.
