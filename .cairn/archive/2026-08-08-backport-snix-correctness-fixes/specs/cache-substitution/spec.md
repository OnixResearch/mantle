## ADDED Requirements

### Requirement: Requested remote PathInfo identity matches the request

r[cache_substitution.requested_path_identity] Mantle MUST require the store-path digest in returned remote PathInfo to equal the requested store-path digest before it accepts or persists the response. A valid signature for a different store path MUST NOT satisfy the request.

#### Scenario: Matching signed PathInfo continues through ordinary admission

GIVEN a remote candidate returns PathInfo whose store-path digest equals the requested digest
AND the response satisfies the selected candidate trust policy
WHEN Mantle performs remote substitution
THEN Mantle MUST continue through ordinary signature, prefix, content, castore, and sidecar admission
AND the digest match alone MUST NOT bypass any later admission rule.

#### Scenario: Signed metadata for another path fails before mutation

GIVEN a remote candidate returns cryptographically valid PathInfo for a store path whose digest differs from the requested digest
WHEN the Snix HTTP service or Mantle substitution shell evaluates the response
THEN Mantle MUST return a stable request-identity mismatch error
AND it MUST NOT persist PathInfo, mutate castore state, create sidecars, export output content, register a root, publish an advisory hit, or report successful substitution.

#### Scenario: Alternative PathInfo service cannot bypass final admission

GIVEN a test or alternative PathInfo service returns PathInfo for a digest other than the requested digest
WHEN Mantle reaches its local remote-substitution finalization boundary
THEN Mantle MUST reject the response before local persistence
AND the service implementation’s own request checks MUST NOT be the only identity guard.

### Requirement: Binary-cache transport normalization is complete

r[cache_substitution.transport_normalization] Mantle MUST normalize binary-cache base URLs as directory bases before endpoint joins and MUST consume every frame of a supported compressed cache payload. Transport normalization MUST NOT weaken existing credential, trust, size, digest, or content-admission rules.

#### Scenario: Equivalent cache bases preserve the configured path

GIVEN two equivalent binary-cache base URLs differ only by a trailing slash
WHEN Mantle derives narinfo, NAR, or supported cache endpoints from each base
THEN both bases MUST produce the same endpoint under the configured base path
AND joining an endpoint MUST NOT replace the final configured path segment.

#### Scenario: Concatenated zstd frames decode completely

GIVEN a supported cache payload contains multiple valid concatenated zstd frames within configured bounds
WHEN Mantle decodes the payload
THEN it MUST consume and validate every frame
AND downstream digest and content checks MUST cover the complete decoded payload.

#### Scenario: Invalid later zstd frame rejects the payload

GIVEN a supported cache payload starts with one valid zstd frame
AND a later frame is truncated, malformed, or exceeds a configured bound
WHEN Mantle decodes the payload
THEN it MUST reject the complete payload with a stable transport error
AND it MUST NOT accept, persist, or report the valid prefix as a complete artifact.
