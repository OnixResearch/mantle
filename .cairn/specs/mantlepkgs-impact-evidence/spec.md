# Mantlepkgs Impact Evidence Specification

## Purpose

Defines the `mantlepkgs-impact-evidence` capability.

## Requirements

### Requirement: Impact reports bind compatible snapshots

r[mantlepkgs_impact.compatible_snapshots] Mantle MUST bind each impact input to exact catalog, system, store-prefix, conversion-policy, package-record, and observation identities.

#### Scenario: Base and head are compatible

GIVEN base and head snapshots use compatible systems, store prefixes, policies, schemas, and identity domains
WHEN the impact core validates both snapshots
THEN it MUST produce one normalized comparison input
AND host paths, input order, and ambient state MUST NOT affect its identity.

#### Scenario: Global facts are incompatible

GIVEN the snapshots use incompatible systems, store prefixes, conversion policies, schemas, or identity domains
WHEN the impact core validates them
THEN it MUST reject the comparison with ordered reason codes
AND it MUST NOT emit numeric package or closure regression claims.

### Requirement: Catalog dispositions are deterministic

r[mantlepkgs_impact.catalog_dispositions] Mantle MUST classify every bounded package and variant key as added, removed, unchanged, recipe changed, policy changed, blocker changed, or variant changed.

#### Scenario: Equivalent records arrive in different orders

GIVEN equivalent package records arrive in different input orders
WHEN the impact core joins base and head records
THEN every disposition and the report BLAKE3 identity MUST match
AND map iteration order MUST NOT affect output order.

#### Scenario: One side has conflicting records

GIVEN a snapshot contains duplicate keys with different recipe, policy, blocker, root, or variant facts
WHEN the impact core normalizes that snapshot
THEN it MUST reject the snapshot with a stable conflict reason
AND it MUST NOT choose one record by input order.

### Requirement: Build transitions preserve missing observations

r[mantlepkgs_impact.outcome_transitions] Mantle MUST derive build transitions only from admitted observations and MUST preserve blocked, not-attempted, unavailable, and missing states.

#### Scenario: Head changes from success to failure

GIVEN compatible admitted base success and head failure observations exist for one package action
WHEN the impact core compares the observations
THEN it MUST report a newly failed transition
AND it MUST bind both observation identities and terminal reason codes.

#### Scenario: Head was not built

GIVEN the base has an admitted success and the head has no admitted build observation
WHEN the impact core compares the package
THEN it MUST report not attempted or unavailable from explicit supplied facts
AND it MUST NOT report newly failed, fixed, or successful.

### Requirement: Closure deltas require complete comparable facts

r[mantlepkgs_impact.closure_deltas] Mantle MUST compute closure deltas only for complete closures with matching system, store prefix, closure semantics, and byte semantics.

#### Scenario: Complete closures are comparable

GIVEN base and head closures have complete member identities, references, and logical byte facts under matching semantics
WHEN the impact core compares them
THEN it MUST report added and removed members, member-count delta, logical byte delta, and retained dependencies
AND checked arithmetic MUST reject overflow.

#### Scenario: Closure facts are incomplete

GIVEN one closure has a missing member, missing size, unresolved reference, mismatched semantic, or exceeded limit
WHEN the impact core evaluates closure comparison
THEN it MUST emit a stable non-comparability reason
AND it MUST NOT substitute zero, truncate members, or emit a numeric regression claim.

### Requirement: Forge integration stays external

r[mantlepkgs_impact.external_ci_boundary] Mantle MUST emit a bounded `mantle-package-impact-v1` report without reading forge credentials, processing webhooks, approving changes, or publishing comments.

#### Scenario: External adapter consumes a report

GIVEN Mantle emitted a valid impact report
WHEN an external CI adapter renders a status, comment, metric, or dashboard
THEN the adapter MUST consume only the contracted report and explicit external configuration
AND its network result MUST NOT change the Mantle report identity.

#### Scenario: Forge configuration is absent

GIVEN no forge token, repository setting, or network is available
WHEN Mantle computes the impact report
THEN report generation MUST remain available from local contracted artifacts
AND it MUST NOT search ambient credential stores.

### Requirement: Impact decisions use a functional core

r[mantlepkgs_impact.functional_core] Mantle MUST keep compatibility validation, package joining, disposition, transition, closure comparison, diagnostic ordering, and report identity construction pure and deterministic.

#### Scenario: Input exceeds a named limit

GIVEN package, variant, closure-member, edge, observation, diagnostic, or report-size input exceeds typed policy
WHEN the core validates the input
THEN it MUST return a typed limit failure before unbounded work
AND the shell MUST NOT publish a truncated complete report.

#### Scenario: Shell observation differs from the plan

GIVEN the shell reads an artifact whose digest or identity differs from the core input
WHEN pre-publication validation runs
THEN Mantle MUST reject publication with a deterministic mismatch
AND the core MUST NOT read the filesystem, store, network, clock, or environment.

### Requirement: Impact-report claims stay bounded

r[mantlepkgs_impact.claim_boundary] Mantle MUST limit impact claims to exact compared snapshots, admitted observations, closure facts, policies, systems, and report identities.

#### Scenario: Candidate impact is favorable

GIVEN changed packages build and comparable closure sizes do not increase
WHEN Mantle writes the impact summary
THEN it MAY report those exact observations
AND it MUST NOT claim package correctness, unchanged behavior, reproducibility, deployment safety, or release eligibility.
