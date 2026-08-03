# Mantlepkgs Update Plans Specification

## Purpose

Define typed package-update policy, bounded source observations, deterministic candidate selection, safe mutation, advisory evidence, and linked validation for Mantlepkgs.

## ADDED Requirements

### Requirement: Update policy is typed and non-executable

r[mantlepkgs_updates.typed_policy] Mantle MUST define package-update policy in typed Nickel and MUST bind its version, package selectors, current source facts, version rules, patch policy, advisory policy, validation policy, and named limits.

#### Scenario: Complete update policy is accepted

GIVEN a policy has exact package and variant selectors, source kind, current version and identity, bounded rules, and safe paths
WHEN Mantle normalizes the policy
THEN it MUST produce one deterministic policy identity
AND record order, comments, host paths, and ambient state MUST NOT change that identity.

#### Scenario: Policy contains executable behavior

GIVEN a policy contains a shell fragment, ambient credential lookup, unbounded rule, unsafe path, floating source, or unknown field
WHEN Mantle normalizes the policy
THEN it MUST reject the policy with ordered diagnostics
AND it MUST NOT perform network, producer, build, or mutation effects.

### Requirement: Source adapters preserve observation status

r[mantlepkgs_updates.source_observations] Mantle MUST record bounded source observations with adapter identity, query, source authority, response identity, candidates, collection facts, and explicit success, unavailable, or failed status.

#### Scenario: Source returns bounded candidates

GIVEN a declared adapter receives a valid bounded response from its configured source authority
WHEN the shell records the observation
THEN it MUST bind the query, adapter, source, response, candidate set, and status
AND saved observation replay MUST NOT require the network.

#### Scenario: Source request fails

GIVEN the request times out, exceeds a bound, returns an invalid status, follows a forbidden redirect, or has an invalid schema
WHEN the shell records the result
THEN it MUST retain unavailable or failed status with a stable reason
AND it MUST NOT replace the result with an empty successful candidate set.

### Requirement: Candidate selection is deterministic

r[mantlepkgs_updates.candidate_selection] Mantle MUST select update candidates through a pure deterministic core over normalized policy and recorded observations.

#### Scenario: Multiple valid versions exist

GIVEN recorded candidates contain several versions that satisfy normalization, allowed ranges, ignored ranges, and prerelease policy
WHEN the core selects a candidate
THEN it MUST return the highest policy-ordered candidate and ordered reason codes
AND input order MUST NOT affect the result.

#### Scenario: No candidate is eligible

GIVEN all recorded candidates are malformed, ignored, disallowed, prerelease, duplicate, ambiguous, or outside policy
WHEN the core selects a candidate
THEN it MUST return a typed no-candidate result
AND it MUST NOT fall back to an unrecorded version or current network state.

### Requirement: Mutation is dry-run and preimage-bound

r[mantlepkgs_updates.preimage_bound_mutation] Mantle MUST default to a no-mutate update plan and MUST bind each proposed structured edit to exact old and new values plus input and output digests.

#### Scenario: Explicit execution has matching preimages

GIVEN an accepted plan and source files whose current digests and structured fields match every planned preimage
WHEN the operator requests execution
THEN the shell MUST write a private stage, validate all outputs, and publish the complete update without clobber
AND the committed receipt MUST bind every applied effect and final digest.

#### Scenario: One preimage changed

GIVEN one target file, field, old value, path type, or digest differs from the accepted plan
WHEN the operator requests execution
THEN the shell MUST reject the complete mutation before final publication
AND it MUST leave the original source files unchanged.

### Requirement: Advisory evidence preserves unavailable data

r[mantlepkgs_updates.advisory_evidence] Mantle MUST record OSV and Repology observations as separate bounded evidence with service, query, package coordinate, version, response digest, schema, status, and results.

#### Scenario: Service reports no findings

GIVEN a successful authenticated response matches the requested package coordinate and version and contains no findings
WHEN Mantle records the observation
THEN it MAY record an empty finding set with successful status
AND it MUST retain the response identity and query facts.

#### Scenario: Service is unavailable or invalid

GIVEN an advisory request has a timeout, transport error, invalid status, schema error, coordinate mismatch, digest mismatch, or exceeded limit
WHEN Mantle records the observation
THEN it MUST retain unavailable or failed status with a stable reason
AND it MUST NOT record zero findings as a substitute.

### Requirement: Candidate evidence links ordinary validation boundaries

r[mantlepkgs_updates.validation_evidence] Mantle MUST link each proposed candidate to its catalog generation, package build observations, separate validation-root observations, package-impact report, and update receipt.

#### Scenario: Candidate has complete favorable evidence

GIVEN a candidate generation has admitted package builds, accepted validation roots, a compatible impact report, and policy-accepted advisory observations
WHEN Mantle derives proposal status
THEN it MAY report the candidate as ready for human review
AND it MUST bind every evidence identity and policy decision.

#### Scenario: Candidate evidence is missing or failed

GIVEN a required build, validation root, impact comparison, or advisory observation is absent, failed, stale, incompatible, or over limit
WHEN Mantle derives proposal status
THEN it MUST return blocked, unavailable, or failed with ordered reasons
AND it MUST NOT report the candidate as fully validated.

### Requirement: Update decisions use a functional core

r[mantlepkgs_updates.functional_core] Mantle MUST keep policy normalization, observation validation, version selection, effect planning, advisory classification, proposal status, diagnostic ordering, and identity construction pure and deterministic.

#### Scenario: Equivalent saved observations replay

GIVEN equivalent policy and saved observations arrive in different orders
WHEN the core derives update plans
THEN plans, effects, reasons, and BLAKE3 identities MUST match
AND the core MUST NOT read files, access networks, inspect the environment, use clocks, run processes, or mutate state.

#### Scenario: Input exceeds a named limit

GIVEN policy, candidate, response, finding, effect, diagnostic, or artifact input exceeds a typed bound
WHEN the core validates the input
THEN it MUST return a typed limit failure before unbounded work
AND the shell MUST NOT continue with truncated success evidence.

### Requirement: Update claims stay bounded

r[mantlepkgs_updates.claim_boundary] Mantle MUST limit update claims to exact policies, source observations, candidate facts, mutations, builds, validation roots, impact reports, advisories, systems, and receipts.

#### Scenario: Candidate is ready for review

GIVEN all required candidate evidence satisfies configured policy
WHEN Mantle writes the update summary
THEN it MAY report readiness for human review under that exact policy
AND it MUST NOT claim source trust, absence of unknown vulnerabilities, package correctness, reproducibility, deployment safety, or release eligibility.
