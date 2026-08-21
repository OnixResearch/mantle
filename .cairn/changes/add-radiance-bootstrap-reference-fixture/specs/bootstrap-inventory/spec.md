## ADDED Requirements

### Requirement: Radiance reference sources form one exact cohort

r[mantle.bootstrap.radiance_reference.source_cohort] Mantle MUST bind exact tagged Git SHA-256 objects, source BLAKE3 values, roles, projections, snapshot profiles, and MIT licenses for Radiance, Radiance.s0, and the emulator.

#### Scenario: Complete source cohort is admitted

- GIVEN all three repositories match their reviewed identities, roles, projections, and licenses
- WHEN source-cohort admission runs
- THEN Mantle MUST produce one deterministic cohort identity

#### Scenario: One source fact differs

- GIVEN a Git format, revision, source byte, role, projection, profile, or license fact differs
- WHEN source-cohort admission runs
- THEN Mantle MUST reject the cohort before build or execution

### Requirement: Reference proof runs offline

r[mantle.bootstrap.radiance_reference.offline] The live reference proof MUST use one authenticated imported source bundle and MUST forbid live source acquisition.

#### Scenario: Offline source closure is complete

- GIVEN the source bundle contains every admitted source and tool input
- WHEN the proof preflight and stages run
- THEN every source request MUST resolve from pinned source state
- AND evidence MUST report zero live fetches and source fallbacks

#### Scenario: Source is absent from the bundle

- GIVEN one required source or tool input is unavailable in pinned source state
- WHEN proof preflight runs
- THEN the proof MUST fail before DNS, proxy, Git, URL, or other network access

### Requirement: Two route roots remain explicit

r[mantle.bootstrap.radiance_reference.build_graph] The fixture MUST define one seed route and one C99 route with distinct admitted roots and one shared Radiance source projection.

#### Scenario: Seed route is planned

- GIVEN the admitted RV64 seed, emulator, source, and limits
- WHEN the seed route is planned
- THEN its first compiler stage MUST name the seed as its immediate predecessor

#### Scenario: C99 route is planned

- GIVEN the admitted host C compiler, Radiance.s0 source, emulator, Radiance source, and limits
- WHEN the C99 route is planned
- THEN it MUST build Radiance.s0 before the first self-hosted compiler stage

### Requirement: Every stage uses its declared predecessor

r[mantle.bootstrap.radiance_reference.lineage] Each compiler stage MUST execute only its declared immediate predecessor under the accepted protected execution policy.

#### Scenario: Stage lineage is complete

- GIVEN each stage names an admitted predecessor, source projection, argv, environment policy, and expected output role
- WHEN protected execution completes
- THEN the audit MUST contain every declared executable and parent-child relation

#### Scenario: Ambient or substituted tool appears

- GIVEN a stage discovers an ambient compiler, skips a predecessor, runs an undeclared executable, substitutes output, or uses fallback
- WHEN execution or audit admission runs
- THEN the proof MUST fail without a fixed-point verdict

### Requirement: Route convergence remains separate from correctness

r[mantle.bootstrap.radiance_reference.convergence] Mantle MUST compare route-local stage-one and stage-two outputs. It MUST record cross-route equality as a separate observation.

#### Scenario: Both routes reach one fixed point

- GIVEN stage one and stage two match inside each route and both route outputs also match
- WHEN convergence classification runs
- THEN the receipt MUST report two route-local fixed points and one cross-route match
- AND it MUST NOT report compiler correctness or seed trust

#### Scenario: Routes converge to different outputs

- GIVEN stage one and stage two match inside each route but the route outputs differ
- WHEN convergence classification runs
- THEN the receipt MUST preserve both fixed points and report cross-route divergence
- AND it MUST NOT identify either route as correct without an external oracle

### Requirement: Reference receipt binds all observed facts

r[mantle.bootstrap.radiance_reference.receipt] The receipt MUST bind source, source-state, toolchain, build, emulator, stage, predecessor, execution, output, comparison, zero-event, and non-claim facts.

#### Scenario: Receipt replays

- GIVEN a completed bounded proof and its exact artifacts
- WHEN receipt validation runs
- THEN it MUST recompute all identities and convergence outcomes from supplied evidence

#### Scenario: Evidence is stale or mutated

- GIVEN source, seed, compiler, emulator, lineage, output, comparison, or receipt bytes change
- WHEN receipt validation runs
- THEN it MUST fail with the first bounded identity or relation mismatch

### Requirement: Published fixtures are immutable and portable

r[mantle.bootstrap.radiance_reference.publication] Selected RV64 programs and compiler artifacts MUST publish through immutable identities without ambient sibling paths.

#### Scenario: Differential consumer selects a fixture

- GIVEN one published fixture has an accepted manifest and receipt
- WHEN another repository consumes it
- THEN the consumer MUST use the published immutable source and artifact identities
- AND the active Mantle worktree MUST NOT become a runtime dependency

### Requirement: Reference claims remain bounded

r[mantle.bootstrap.radiance_reference.claim_boundary] Mantle MUST describe the fixture as external bootstrap evidence for exact inputs and MUST reject stronger correctness or trust claims.

#### Scenario: Operator reviews a passing receipt

- GIVEN both route-local fixed points and a cross-route comparison are valid
- WHEN Mantle renders the result
- THEN it MAY report bounded convergence and lineage facts
- AND it MUST NOT claim compiler correctness, seed trust, semantic equivalence, or universal reproducibility
