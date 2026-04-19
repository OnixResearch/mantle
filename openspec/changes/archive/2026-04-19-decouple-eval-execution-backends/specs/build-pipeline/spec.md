## MODIFIED Requirements

### Requirement: Parallel builds

The build pipeline MUST treat eval root-force backend selection as runtime host
policy that is separate from `crunch-eval` forcing semantics.

The shipped runtime MAY prefer a local threaded eval backend when forcing roots
for build streaming, but it MUST keep the required inline backend as the
fallback when the preferred non-inline backend is unavailable, unsupported, or
not selected for the current request.

The runtime build pipeline MUST preserve root-label association, converted
output association, and labeled eval failure reporting regardless of whether the
current request used the preferred non-inline backend or the inline fallback.

The build pipeline MUST keep backend selection command-scoped. It MUST NOT
introduce a resident eval daemon or require library-mode binary self-spawn in
order to build derivations.

#### Scenario: Runtime root-force preference falls back to inline

- GIVEN the build runtime prefers a local threaded eval backend for root
  forcing
- AND that backend is unavailable, unsupported, or not selected for the current
  request
- WHEN the runtime streams build roots through the eval boundary
- THEN it falls back to the required inline backend for that request
- AND root-label association and converted output association remain unchanged

#### Scenario: Runtime eval failure stays labeled across backend choice

- GIVEN a build run with one root that fails during eval forcing
- WHEN the runtime executes that request through either the preferred
  non-inline backend or the inline fallback
- THEN the final build report identifies the same failed root label
- AND the runtime does not depend on partial-success results from that failed
  request
