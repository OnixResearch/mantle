# Molten SpaceWasm ABI materialization

## ADDED Requirements

### Requirement: Pilot fixtures retain exact authoring and generated identities

r[mantle.spacewasm_pilot.fixtures] Mantle MUST materialize the declared
segmentation and canonical ABI fixture family from retained authoring sources
using the existing pinned generator, validate wasm1, independently read back
binary bytes, and bind each WAT, Wasm and descriptor member by BLAKE3 and size.

#### Scenario: Identical fixture inputs are materialized
- GIVEN admitted authoring sources and the exact tool cohort
- WHEN producer materialization runs
- THEN the generated module and binary readback MUST match
- AND the complete bundle MUST retain every fixture member identity.

#### Scenario: Fixture bytes are missing or substituted
- GIVEN an omitted, modified or hostless-substituted pilot member
- WHEN complete-bundle verification runs
- THEN verification MUST reject the bundle.

### Requirement: Consumer runtime and publication admission remain separate

r[mantle.spacewasm_pilot.boundary] Mantle MUST preserve the existing hostless
reference profile and MUST NOT claim the additional modules executed in its
runner. A pilot consumer MUST require selected member paths and external pinned
producer/publication identity before execution; a self-consistent local build
MUST NOT establish this identity by itself.

#### Scenario: Hostless evidence is presented for a host ABI pilot
- GIVEN an old valid reference bundle without required pilot members
- WHEN the consumer admits its pilot inputs
- THEN it MUST deny the pilot without locally compiling a substitute.

#### Scenario: A complete materialized fixture is consumed
- GIVEN exact member bytes and externally pinned producer evidence
- WHEN consumer-owned ABI, hostcall, memory, trap, stack and cumulative fuel rails run
- THEN their outcomes MUST remain separate from producer materialization facts.
