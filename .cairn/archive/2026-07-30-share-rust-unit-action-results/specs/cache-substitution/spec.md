## ADDED Requirements

### Requirement: Shared Rust unit action-result discovery

r[cache_substitution.rust_unit_action_result_discovery] Mantle MUST support bounded provider-neutral discovery of signed Rust unit result candidates by canonical action reference. Discovery alone MUST NOT authorize reuse.

#### Scenario: Fresh client discovers a reusable Rust unit result

r[cache_substitution.rust_unit_action_result_discovery.clean_client]

GIVEN a producer published complete immutable castore objects, a signed Rust unit result record, and its action index candidate
AND a fresh client has no local unit result or execution output
WHEN the client queries configured result sources for the same canonical action reference
THEN Mantle MAY fetch the bounded candidate records and referenced objects
AND it MUST skip compiler execution only after one candidate passes Rust-specific authority, identity, completeness, artifact, and materialization admission.

#### Scenario: Candidate admission verifies complete authority and content

r[cache_substitution.rust_unit_action_result_discovery.candidate_admission]

GIVEN a remote source returns a Rust unit result candidate
WHEN Mantle evaluates it for reuse
THEN Mantle MUST verify the canonical record identity, full signing key material, producer policy, action reference, result reference, artifact manifest, schema limits, and complete castore tree
AND a signer name, index presence, transport success, or root object presence MUST NOT admit the result by itself.

#### Scenario: Offline policy keeps remote sources unopened

r[cache_substitution.rust_unit_action_result_discovery.offline]

GIVEN offline Rust topology execution is selected
AND no admitted local Rust unit result exists
WHEN Mantle plans shared result discovery
THEN every remote result and object source MUST remain unopened
AND Mantle MUST report an offline remote-cache miss before another eligible local route.

#### Scenario: Publication exposes only complete immutable state

r[cache_substitution.rust_unit_action_result_discovery.publication]

GIVEN Mantle has admitted a new Rust unit result for publication
WHEN it publishes shared state
THEN complete immutable castore objects MUST become available before the signed result record
AND the signed result record MUST become available before its no-clobber action index candidate
AND readers MUST observe either the prior complete candidate set or the new complete candidate set.

#### Scenario: Conflicting shared results remain visible

r[cache_substitution.rust_unit_action_result_discovery.conflicts]

GIVEN multiple signed and otherwise admissible Rust unit result records claim one action reference
AND their artifact sets differ
WHEN Mantle plans strong reuse
THEN Mantle MUST emit deterministic nondeterminism evidence
AND it MUST NOT select by response time, configured source order, insertion order, or last writer.

#### Scenario: Corrupt or unavailable remote state does not fabricate reuse

GIVEN a result source or object source returns malformed, oversized, untrusted, incomplete, corrupt, redirected, timed-out, or unavailable state
WHEN Mantle evaluates the candidate
THEN Mantle MUST reject it with a stable bounded reason
AND it MUST NOT publish a successful output or reuse receipt
AND an eligible later candidate or compiler execution MAY continue only under explicit policy.

#### Scenario: Remote diagnostics are bounded and redacted

GIVEN shared Rust cache configuration contains URLs, credentials, verifier keys, priorities, or several sources
WHEN Mantle emits human or JSON diagnostics
THEN it MUST report only sanitized source identity, public verifier identity, result identity, authority disposition, reason codes, and bounded byte counts
AND it MUST omit credentials, bearer tokens, signed URL queries, private keys, and unbounded raw configuration.
