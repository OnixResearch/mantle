# Wasm Component Effects Specification

## Purpose

Defines the `wasm-component-effects` capability.

## Requirements

### Requirement: Closed static effects cohort

r[mantle.wasm_component.effects_cohort] The producer MUST select `molten-component-effects-v1` only by an exact declared
profile identity in its immutable checker/toolchain cohort. It MUST retain the
portable and Molten import-free baseline profiles without broadening either.

#### Scenario: Unknown or ambiguous profile

- GIVEN a toolchain declaring only its pinned portable, Molten baseline, and effects profiles
- WHEN a request uses an undeclared profile identity or the cohort contains duplicate names/identities
- THEN materialization MUST be rejected without fallback to the default profile

### Requirement: Consumer-owned runtime validation

r[mantle.wasm_component.consumer_host_requirement] A host-import component MAY request consumer-host-required runtime validation.
The request MUST exactly bind WIT package/world and remaining imports and MUST
NOT declare standalone invocation or expected stdout. Its bundle MUST serialize
and identify that policy together with exact artifact/WIT/profile/cohort inputs.

#### Scenario: Static materialization without fake execution

- GIVEN exact reviewed source/WIT bytes and a well-formed consumer-host-required policy
- WHEN the exact host-import artifact passes every source, lock, build-local, Octet, materialization, and byte-verification gate
- THEN the runtime requirement MUST be recorded as not-run with no producer execution claims
- **AND** the verified bundle MUST expose the typed requirement for a real consumer host

#### Scenario: Contradictory runtime policy

- GIVEN a consumer-host request that cannot be fulfilled by standalone Wasmtime
- WHEN a consumer-host request declares standalone invocation/stdout, mismatched WIT/import data, or a producer smoke claim
- THEN admission MUST reject it

### Requirement: Genuine deterministic producer fixtures

r[mantle.wasm_component.effect_fixtures] The effect fixture MUST use exact Molten-owned versioned WIT and pinned producer
bindings without text rewriting, synthetic dependencies, or ambient inputs.
Explicit request modes MUST support pure response, real host forwarding with
propagated results, trap, malformed output, and fuel/deadline workloads.

#### Scenario: Real consumer operates exact artifact

- GIVEN a genuine producer artifact with an unfulfilled consumer-host requirement
- WHEN actor and system-extension consumers load a completely verified effect bundle
- THEN their actual host MUST satisfy the declared host surface before producing product execution receipts or activation
- **AND** static bundle verification MUST NOT be represented as runtime authority or execution

### Requirement: Standalone and empty-closure integrity

r[mantle.wasm_component.standalone_empty_closure] The producer MUST retain standalone-smoke defaults and old canonical bundle
identities. An empty package closure MUST still bind its exact checked lock and
reject invented package materializations or drifted lock bytes.

#### Scenario: Empty closure is not unchecked closure

- GIVEN a dependency-free source and an exact checked empty package lock
- WHEN a dependency-free source declares an exact empty lock
- THEN source acquisition MAY produce no package fetches
- AND altered lock bytes or unexpected package materializations MUST fail admission
