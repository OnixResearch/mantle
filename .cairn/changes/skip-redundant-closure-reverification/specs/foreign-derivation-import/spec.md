## ADDED Requirements

### Requirement: Hydrated inputs are verified once per realization

r[foreign_derivation_import.hydrated_input_verification_once] The cache-inputs build routes MUST
verify each hydrated input against the signed cache in one pass and MUST NOT re-ingest physical
input exports after hydration. A store lookup used for hydrated inputs and for the presence check of
non-root unit outputs MUST return the complete local `PathInfo` with the same castore-completeness
and `PathInfo` identity checks as the exporting lookup, MUST NOT materialize the path, and MUST NOT
record a built output. Selected-root classification, the cache-only route, and route v1 MUST keep
the exporting lookup.

Consequence: on a reuse run the route MUST NOT re-verify or repair the physical exports of inputs.
Builds read inputs from castore, and the selected root keeps its own NAR check in
`classify_present_root`.

#### Scenario: Hydration verifies members once

- GIVEN a signed cache closure whose members were exported and NAR-checked by the closure import
- WHEN `hydrate_signed_cache_closure` walks the members
- THEN every member MUST yield its complete local `PathInfo` without an additional physical export
- AND the resulting hydration facts MUST be unchanged.

#### Scenario: A reuse run does no input re-ingest

- GIVEN a route-v2 realization that reuses a previously built selected root
- WHEN `locally_present_input_paths` checks non-root unit outputs
- THEN it MUST use the lookup-only store access
- AND it MUST NOT ingest or repair any input export.

#### Scenario: Selected roots keep the exporting lookup

- GIVEN a selected root that is present locally on route v2
- WHEN realization classifies it
- THEN the root MUST still be checked and exported through the exporting lookup
- AND the reuse decision MUST be unchanged.

#### Scenario: An unknown path is absent

- GIVEN a store path with no complete local content
- WHEN the lookup-only access is called
- THEN it MUST return `None`.
