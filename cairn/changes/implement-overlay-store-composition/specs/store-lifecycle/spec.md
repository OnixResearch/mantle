# Store Lifecycle Overlay Delta

## ADDED Requirements

### Requirement: Ordered overlay composition

r[store_lifecycle.overlay_composition] Mantle MUST support one writable overlay above a bounded ordered list of read-only base stores. Every layer MUST use the same logical store prefix, and reads MUST use overlay-first declaration order while all mutations target only the overlay.

#### Scenario: Base satisfies an overlay miss

- **GIVEN** the overlay lacks a valid requested value and the first eligible base contains it under the shared logical prefix
- **WHEN** composed lookup runs
- **THEN** Mantle MUST return the base value with its layer provenance
- **AND** it MUST NOT insert the value into the overlay

#### Scenario: Logical prefixes differ

- **GIVEN** an overlay and declared base use different logical store prefixes
- **WHEN** Mantle validates composition
- **THEN** it MUST reject the configuration before composed services open
- **AND** it MUST NOT rewrite paths or combine the stores as independent namespaces

### Requirement: Overlay reads do not backfill

r[store_lifecycle.overlay_no_backfill] Base PathInfo, directory, and blob reads in overlay mode MUST NOT copy content or metadata into the writable overlay. Existing cache combinators MAY retain backfill only outside overlay mode.

#### Scenario: Repeated base read stays thin

- **GIVEN** a valid value exists only in a read-only base
- **WHEN** Mantle reads it repeatedly through the composed view
- **THEN** every read MUST resolve from the base
- **AND** overlay PathInfo, directory, blob, and output state MUST remain unchanged

#### Scenario: Cache mode is selected instead

- **GIVEN** an existing caller selects ordinary cache-with-backfill mode
- **WHEN** the far service satisfies a miss
- **THEN** the current backfill behavior MAY remain
- **AND** the caller MUST NOT label that behavior as overlay composition

### Requirement: Base descriptors bind generation facts

r[store_lifecycle.overlay_base_generation] Every base MUST have a deterministic descriptor that binds logical prefix, state schema, trust policy, read-only capability, ordered precedence, and bounded base generation facts. Plans and admitted read decisions MUST reject relevant base generation drift.

#### Scenario: Base remains unchanged

- **GIVEN** planning and execution observe matching base descriptor and generation facts
- **WHEN** a selected build consumes base content
- **THEN** the route and build reports MUST bind the base descriptor and selected layer
- **AND** the claim MUST remain limited to the observed base generation

#### Scenario: Base changes after planning

- **GIVEN** relevant base PathInfo, root inventory, content metadata, descriptor, or trust-policy facts change after planning
- **WHEN** execution rechecks the bound generation
- **THEN** Mantle MUST reject the stale plan or read decision before output admission
- **AND** it MUST NOT claim a whole-database snapshot when the backend did not provide one

### Requirement: Layer-local trust and fail-closed precedence

r[store_lifecycle.overlay_layer_trust] Mantle MUST evaluate PathInfo, content, signatures, attestations, and policy under the selected layer. A higher-precedence invalid value MUST block lookup by default and MUST NOT borrow trust from or silently fall through to a lower layer.

#### Scenario: Trusted overlay shadows trusted base

- **GIVEN** overlay and base contain the same logical path and the overlay record passes overlay trust and content admission
- **WHEN** composed lookup resolves the path
- **THEN** Mantle MUST select the overlay and report the shadow relationship in bounded form
- **AND** base signatures MUST NOT become overlay trust evidence

#### Scenario: Overlay shadow is corrupt

- **GIVEN** the overlay contains a higher-precedence path with corrupt, incomplete, prefix-mismatched, or untrusted facts and a lower base contains a valid path
- **WHEN** composed lookup validates the overlay value
- **THEN** it MUST fail with a layer-specific blocker under default policy
- **AND** it MUST NOT silently select the lower base

### Requirement: Overlay write isolation

r[store_lifecycle.overlay_write_isolation] Every store mutation in overlay mode MUST target overlay-owned services and state. Mantle MUST NOT write, repair, sign, root, collect, backfill, or publish action results into a declared base.

#### Scenario: Build writes an output

- **GIVEN** a build consumes base inputs and produces a new output
- **WHEN** Mantle persists output content, PathInfo, roots, attestations, and action-result metadata
- **THEN** every mutation MUST occur in the overlay
- **AND** base write sentinels MUST observe no attempted mutation

#### Scenario: Base mutation seam is reached

- **GIVEN** a repair, signing, substitution, root, GC, or persistence path attempts to use a base write handle
- **WHEN** the read-only capability rejects the operation
- **THEN** Mantle MUST fail without retrying the write through another base path
- **AND** it MUST not report the requested mutation as complete

### Requirement: Cross-layer GC safety

r[store_lifecycle.overlay_gc_safety] Overlay GC MUST plan reachability through the composed read view and MUST remove only overlay-owned unreferenced state. Base-satisfied references MUST remain read-through references and MUST NOT trigger backfill or base mutation.

#### Scenario: Overlay root reaches base content

- **GIVEN** an overlay root closure includes valid paths and objects satisfied by a base
- **WHEN** overlay GC plans collection
- **THEN** it MUST retain required overlay-owned members and record base-satisfied reachability
- **AND** it MUST neither copy nor delete the base content

#### Scenario: Base refers to overlay-only state

- **GIVEN** base PathInfo or directory facts require a path or object found only in the writable overlay
- **WHEN** composition or GC validation observes that reverse dependency
- **THEN** Mantle MUST reject it as an unsupported base-to-overlay ownership relation
- **AND** it MUST not keep the overlay object through hidden base authority

### Requirement: Composed sandbox and inspection view

r[store_lifecycle.overlay_sandbox_view] Build source resolution, closure walking, sandbox input access, store inspection, graph queries, and attestation lookup MUST consume one composed read interface without exposing physical layer layout as recipe meaning.

#### Scenario: Sandbox consumes base-only input

- **GIVEN** a selected derivation input is valid and available only in a base
- **WHEN** Mantle resolves and mounts the input for a build
- **THEN** the sandbox MUST receive the logical input through the ordinary composed castore path
- **AND** derivation identity MUST remain independent from the physical base location

#### Scenario: Composed input is incomplete

- **GIVEN** PathInfo resolves from one layer but required directory or blob content is missing or invalid under composed policy
- **WHEN** source or sandbox preparation runs
- **THEN** Mantle MUST fail before builder start
- **AND** it MUST identify the selected layer and missing content class without exposing private physical paths

### Requirement: Overlay validation

r[store_lifecycle.overlay_validation] Overlay composition MUST include positive, negative, no-backfill, write-sentinel, trust, generation-drift, cross-layer-GC, sandbox, race, and single-store-parity fixtures before acceptance.

#### Scenario: Supported overlay matrix passes

- **GIVEN** valid single-base, multi-base, overlay-shadow, base-closure, overlay-write, GC, and sandbox fixtures
- **WHEN** focused overlay validation runs
- **THEN** every fixture MUST produce its expected layer and mutation decisions
- **AND** single-store mode MUST retain current behavior when no base is declared

#### Scenario: Unsafe overlay fixture rejects

- **GIVEN** a fixture has prefix drift, corrupt precedence, untrusted shadow, writable base, generation drift, reverse dependency, write attempt, or exceeded bound
- **WHEN** focused overlay validation runs
- **THEN** it MUST reject with the expected stable class
- **AND** an unrelated failure MUST NOT count as correct fail-closed evidence
