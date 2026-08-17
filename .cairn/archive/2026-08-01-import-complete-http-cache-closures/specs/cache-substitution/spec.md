## ADDED Requirements

### Requirement: Complete HTTP cache closure pull is metadata-first and bounded

r[cache_substitution.complete_http_closure_pull] Mantle MUST provide an explicit HTTP cache closure-pull mode that discovers and validates the complete signed narinfo reference graph for one selected root before it downloads NAR content. The mode MUST preserve exact foreign store identities, MUST bind the cache authority, trust policy, logical store prefix, root, limits, and canonical member facts into a BLAKE3 plan identity, and MUST admit the selected root only after every dependency is locally complete.

#### Scenario: Complete signed closure is imported

GIVEN one explicit HTTP cache root has a finite signed narinfo reference graph within configured limits
AND every member satisfies the selected cache trust policy and store-prefix policy
WHEN the operator requests closure pull
THEN Mantle MUST discover the complete metadata graph before the first NAR request
AND it MUST import or reuse every dependency before it imports the selected root.

#### Scenario: Existing explicit pull remains non-recursive

GIVEN an HTTP narinfo declares references
WHEN the operator requests ordinary pull without closure mode
THEN Mantle MUST preserve the existing explicit-path scope
AND it MUST NOT expand network or mutation scope to referenced paths implicitly.

#### Scenario: Missing dependency blocks root admission

GIVEN the selected root narinfo names a dependency whose narinfo is missing, malformed, untrusted, conflicting, or outside a configured bound
WHEN Mantle discovers the closure
THEN it MUST fail before any NAR download
AND it MUST NOT persist, export, or report the selected root as admitted.

#### Scenario: Dependency content failure keeps root absent

GIVEN complete metadata discovery succeeds
AND a dependency NAR is missing, malformed, hash-invalid, or inconsistent with the planned member facts
WHEN Mantle imports the planned closure
THEN it MAY retain previously admitted dependency content as resumable cache state
AND it MUST NOT persist, export, or report the selected root as admitted.

#### Scenario: Incomplete local member is fetched again

GIVEN local PathInfo exists for a planned member
BUT its full castore content is incomplete
WHEN Mantle imports the planned closure
THEN it MUST treat that member as missing and fetch it through ordinary admission
AND PathInfo presence alone MUST NOT satisfy closure completeness.

#### Scenario: Resource limit fails without panic

GIVEN remote metadata exceeds the configured member, depth, narinfo-byte, aggregate NAR-size, or duplicate/conflict bounds
WHEN the pure planner or HTTP discovery shell evaluates the metadata
THEN Mantle MUST return a stable bounded error without panic or silent truncation
AND it MUST NOT start a NAR request or report a successful closure plan.

#### Scenario: Successful closure import preserves non-claims

GIVEN Mantle imports every planned member and the selected root
WHEN it reports the result
THEN the report MUST identify the plan BLAKE3, member count, imported count, reused count, and selected root state
AND it MUST NOT claim package correctness, local rebuild compatibility, evaluator parity, reproducibility, private-cache authentication, or release eligibility.
