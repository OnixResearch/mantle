# Cache Substitution Specification

## Purpose

Define bounded local and remote discovery and publication of immutable action-result records.

## ADDED Requirements

### Requirement: Shared action-result discovery is bounded and advisory
r[cache_substitution.shared_action_result_discovery]

Mantle MUST support provider-neutral lookup of a bounded candidate set by canonical action ref from configured local or remote result sources. Lookup and publication policy MUST be typed and explicit. Index presence, source authority, transport success, or candidate count MUST NOT admit an output; candidates MUST pass build-correctness reuse admission before execution is skipped.

#### Scenario: Fresh client discovers reusable action result

- GIVEN a producer atomically published a complete immutable action-result record and its referenced admitted artifacts
- AND a fresh client has no local CA derivation mapping for the action
- WHEN the client queries a configured result source by action ref
- THEN Mantle MAY discover and fetch the bounded candidate records and referenced objects
- AND it MUST skip execution only after one candidate passes ordinary reuse admission.

#### Scenario: Offline planning performs no remote lookup

- GIVEN offline policy is selected and no admitted local result exists
- WHEN Mantle plans action-result discovery
- THEN every remote result source MUST remain unopened
- AND the plan MUST report an offline miss or another eligible local route without fabricating shared reuse.

#### Scenario: Publication is atomic and no-clobber

- GIVEN Mantle has a newly admitted result for an action ref
- WHEN it publishes the result record and index candidate
- THEN readers MUST observe either the prior complete candidate set or the new complete candidate set
- AND a partial write, duplicate write, or differing existing candidate MUST NOT erase another immutable candidate or expose missing referenced evidence as complete.

#### Scenario: Unbounded or corrupt index fails closed

- GIVEN a result source returns too many candidates, oversized metadata, malformed refs, duplicate-conflicting records, invalid signatures, or bytes inconsistent with declared record identity
- WHEN Mantle validates discovery
- THEN it MUST reject the offending source response with stable diagnostics
- AND it MUST NOT allocate beyond configured bounds, execute source-provided commands, or mutate admitted store state.
