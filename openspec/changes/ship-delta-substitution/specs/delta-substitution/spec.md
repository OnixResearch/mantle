## ADDED Requirements

### Requirement: Trusted HTTP substituters may satisfy cache hits through delta mode

The substitution pipeline MUST allow a trusted HTTP substituter to satisfy a
cache hit through delta transfer instead of a whole-artifact fetch.

Delta negotiation MUST use the same trusted cache authority that would serve the
ordinary substitution request. If the cache does not support delta mode, crunch
MUST fall back to the ordinary full-artifact substitution path.

#### Scenario: Delta-capable cache hit uses reusable local content

- GIVEN a trusted HTTP substituter that supports delta transfer
- AND the receiver has reusable local content for the requested output
- WHEN crunch requests that output from the cache
- THEN crunch may fetch only the missing content through delta mode
- AND the final result is accepted as an ordinary cache hit after normal final
  verification succeeds

#### Scenario: Legacy cache still serves a normal cache hit

- GIVEN a trusted HTTP substituter that does not support delta transfer
- WHEN crunch requests that output from the cache
- THEN crunch uses the ordinary full-artifact substitution path
- AND the request does not fail merely because delta support is absent

### Requirement: Receiver compatibility manifests are bounded and local-fact based

Crunch MUST build receiver compatibility manifests from local `PathInfo` and
local castore presence only.

The manifest MUST be scoped to the requested output or requested closure. Local
metadata without backing castore content MUST be treated as absent from the
manifest.

#### Scenario: Missing backing content is not advertised as reusable

- GIVEN local `PathInfo` exists for an output but the referenced castore content
  is missing locally
- WHEN crunch builds the receiver compatibility manifest
- THEN that content is not advertised as reusable
- AND delta planning treats it as absent

#### Scenario: Manifest scope stays bounded to the request

- GIVEN a receiver store much larger than the requested closure
- WHEN crunch builds the receiver compatibility manifest
- THEN the manifest covers only the requested output or closure scope
- AND crunch does not enumerate the entire local store first

### Requirement: Final acceptance matches ordinary substitution semantics

A delta transfer MUST be accepted only after crunch reconstructs the final
result, verifies the final signed `PathInfo`, and records the same local
metadata and attestation outcome required for an ordinary substitution cache
hit.

#### Scenario: Delta transport does not bypass trust checks

- GIVEN a delta stream completes successfully
- BUT the final `PathInfo` signatures do not match any trusted key
- WHEN crunch verifies the result
- THEN the cache hit is rejected
- AND crunch does not treat successful transport alone as acceptance

#### Scenario: Delta-backed substitution records ordinary attestation outcome

- GIVEN a delta-backed substitution succeeds and is accepted as a cache hit
- WHEN crunch finalizes that result locally
- THEN it records the same local metadata and artifact-attestation outcome as
  the ordinary full-artifact substitution path

### Requirement: Substitution reporting names delta reuse explicitly

Crunch MUST report whether a successful substitution used `delta` or `full`
transfer mode.

For successful `delta` and successful `full` substitutions, reporting MUST
include transferred-byte accounting. Reporting MUST include reused-byte
accounting whenever local reuse was part of the completed transfer. Reporting
MUST include an explicit fallback reason only when crunch started delta
negotiation but finished through full-artifact fetch instead.

#### Scenario: JSON build report includes delta reuse stats

- GIVEN a successful substitution that used delta transfer
- WHEN crunch emits the JSON build report
- THEN the report includes substitution mode `delta`
- AND it includes transferred-byte and reused-byte counts

#### Scenario: Fallback reason is visible after delta negotiation fails

- GIVEN crunch starts delta negotiation but falls back to ordinary substitution
- WHEN the final substitution succeeds through full fetch
- THEN reporting names transfer mode `full`
- AND it records the reason delta mode was abandoned
