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
