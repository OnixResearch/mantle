## MODIFIED Requirements

### Requirement: Pipeline owns eval and convert

The pipeline MUST continue to evaluate Nickel through `crunch-eval`, discover
root labels through the lazy session boundary, force derivation values per root
only when needed for conversion, and use direct typed derivation extraction
rather than a whole-program JSON export string for build execution.

The pipeline MUST treat eval execution strategy as host policy layered on top of
those `crunch-eval` semantics. It MAY choose or receive a local inline,
threaded, or subprocess backend, but it MUST NOT make one specific thread-pool
or subprocess mechanism a semantic requirement of `crunch-eval` itself.

The pipeline MUST preserve root-label association, conversion semantics, and
failure reporting across every shipped eval execution backend.

For streaming execution, the pipeline MUST treat each root-force request as an
independent error-handling unit. It MUST NOT depend on partial success results
from a failed multi-root request.

#### Scenario: Pipeline result semantics stay stable across eval backends

- GIVEN a `.ncl` file exporting multiple root derivations
- WHEN `build()` runs with different shipped eval execution backends
- THEN the discovered root labels remain the same
- AND the converted derivations remain associated with the same labels
- AND a root-force failure reports the same failed label regardless of backend

#### Scenario: Pipeline may choose a host backend without changing eval-core contract

- GIVEN the current runtime prefers a local threaded or subprocess backend for
  performance on one host
- WHEN `build()` invokes `crunch-eval`
- THEN the pipeline may use that host policy choice
- AND the portable eval-core contract still remains valid with the required
  inline backend alone
