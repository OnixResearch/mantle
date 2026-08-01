# Cache Substitution Specification

## Purpose

Defines the `cache-substitution` capability.

## Requirements

### Requirement: Ordered substituter candidates

r[cache_substitution.ordered_substituters] Mantle MUST model configured substituters as a bounded ordered set of cache candidates. Candidate selection MUST use deterministic configured priority and explicit tie-breakers, MUST keep cache identity and trust-policy facts separate per candidate, and MUST NOT select a remote cache solely because its probe response arrived first.

#### Scenario: priority decides between equivalent cache hits

GIVEN two trusted substituters both advertise an acceptable hit for the same requested output
AND their cache-candidate facts are otherwise equivalent except for configured priority
WHEN Mantle plans or performs substitution
THEN Mantle MUST select the higher-priority candidate deterministically
AND the report MUST identify the selected candidate by a sanitized cache identity rather than raw secret-bearing configuration.

#### Scenario: trust material is not mixed between caches

GIVEN two substituters have the same public key name but different key material or trust-policy digests
WHEN Mantle verifies a remote PathInfo or delta result
THEN Mantle MUST verify it against the trust facts for the candidate that supplied the metadata or content
AND it MUST NOT accept a signature by name alone from a different candidate.

#### Scenario: offline policy rejects network-required candidates

GIVEN offline realization policy is selected
AND a substituter candidate would require live remote metadata, capability, or content lookup
WHEN Mantle plans cache substitution
THEN Mantle MUST reject that candidate before network access
AND diagnostics MUST include a stable offline-network-required reason code.

### Requirement: Advisory remote metadata cache

r[cache_substitution.remote_metadata_cache] Mantle MUST persist and reuse remote cache metadata only as advisory discovery data. Cached metadata MAY cover narinfo or reference presence, negative misses, cache preflight, and delta capability facts, but final output acceptance MUST still verify signed PathInfo, store prefix, content hashes, castore completeness, and required attestation side effects at the store finalization boundary.

#### Scenario: cached positive metadata avoids repeat discovery

GIVEN Mantle has a fresh advisory metadata record showing that a trusted remote candidate has PathInfo references for a requested output
WHEN Mantle plans the same output under an equivalent cache identity, trust policy, store prefix, output digest, metadata class, and metadata schema version
THEN Mantle MAY use the cached metadata record to avoid repeating the remote discovery probe
AND the plan MUST report that the remote availability fact came from advisory metadata.

#### Scenario: cached metadata cannot admit output by itself

GIVEN a fresh metadata record says a remote candidate is available
WHEN Mantle performs substitution for the requested output
THEN Mantle MUST still verify the returned PathInfo signature, logical store prefix, content hash, castore content, and required sidecars before reporting a cache hit
AND it MUST reject or fall back if final verification fails.

#### Scenario: stale or mismatched metadata is ignored

GIVEN an advisory metadata record is expired, has an unsupported schema version, was keyed under a different trust-policy digest, was keyed under a different logical store prefix, or is bypassed by explicit refresh policy
WHEN Mantle plans or performs substitution
THEN Mantle MUST ignore or refresh that record
AND diagnostics MUST distinguish metadata refresh from final cache miss.

### Requirement: Structured cache admission diagnostics

r[cache_substitution.structured_admission_diagnostics] Mantle MUST expose bounded structured cache admission diagnostics for every requested output. Diagnostics MUST use stable reason codes for local hits, local misses, remote hits, remote misses, trust rejection, prefix mismatch, fixed-output local-build policy, delta fallback, metadata-cache reuse, and castore incompleteness, and JSON output MUST remain parseable and redacted.

#### Scenario: untrusted local metadata explains rebuild

GIVEN local PathInfo exists for an output but none of its signatures are trusted under the selected policy
WHEN Mantle renders `build --plan` or `--json build`
THEN the output diagnostics MUST include a stable untrusted-PathInfo reason code
AND the report MUST NOT claim the output was cached.

#### Scenario: delta fallback is visible

GIVEN delta negotiation starts for a remote cache candidate
AND Mantle falls back to full-artifact substitution
WHEN Mantle reports the successful cache hit
THEN the diagnostics MUST include the delta fallback reason
AND transferred and reused byte counts MUST remain bounded numeric fields.

#### Scenario: diagnostics are redacted and bounded

GIVEN cache configuration includes trusted keys, query-string trust material, bearer-capable URLs, or multiple remote candidates
WHEN Mantle emits human or JSON cache diagnostics
THEN it MUST omit secret material and unbounded raw configuration strings
AND it MAY include sanitized cache identities, public verifier names, counts, reason codes, and policy digests.

### Requirement: Complete castore content before local cache hit

r[cache_substitution.castore_completeness] Mantle MUST require complete backing castore content before accepting a local cache hit. A local PathInfo whose root node exists but whose referenced child directories or blobs are missing MUST be rejected as a cache miss with a stable castore-incomplete reason before Mantle reports cached output reuse.

#### Scenario: missing child content rejects local hit

GIVEN local PathInfo exists for an output whose root directory node is present
BUT at least one child directory or blob referenced by that root is missing from local castore storage
WHEN Mantle checks the local cache
THEN Mantle MUST reject the local hit as castore-incomplete
AND it MUST fall back to an eligible archive, trusted substituter, source-bundle route, remote builder, or local build according to policy.

#### Scenario: immutable completeness marker permits cheap repeat hit

GIVEN Mantle has already verified the complete castore tree for an immutable finalized node identity
WHEN an equivalent local cache check repeats
THEN Mantle MAY use a persisted completeness marker instead of recursively probing every child again
AND the marker MUST be keyed by finalized node identity and invalidated or ignored if the marker schema or store prefix facts do not match.

#### Scenario: symlink content is inline

GIVEN a local PathInfo root is a symlink node with an inline target
WHEN Mantle checks local cache completeness
THEN the symlink root MAY be considered complete without blob or directory probes
AND the cache hit still MUST satisfy signature, prefix, and PathInfo identity checks.

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
