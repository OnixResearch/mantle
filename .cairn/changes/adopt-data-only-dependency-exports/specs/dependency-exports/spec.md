# Specification: Data-only dependency exports

## ADDED Requirements

### Requirement: Typed exports record

r[mantle.dependency_exports.exports_record] Each adopted output MUST carry a
typed exports record naming its include directories, library directories,
pkg-config directories, environment defaults, and propagated dependencies.

The record MUST be derived from the output tree by default, with explicit
overrides allowed. Environment values MUST use an output-relative placeholder
instead of absolute store paths where the value denotes a path inside a
dependency. Records MUST be bounded in size and key count.

#### Scenario: Consumer renders search paths from records

- GIVEN two adopted outputs with exports records
- WHEN a consumer's prepare step renders its environment
- THEN include paths, library paths, and pkg-config paths MUST come only from
  the records.

#### Scenario: Placeholder resolves per consumer

- GIVEN an environment default containing an output-relative placeholder
- WHEN the consumer expands it
- THEN the expanded value MUST point inside the named dependency output
  without an absolute build-time path.

### Requirement: Dependencies contribute no behavior

r[mantle.dependency_exports.no_behavior_hooks] Adopted families MUST NOT
execute dependency-provided code in consumer builds.

No dependency MAY inject phases, hooks, or shell fragments into a consumer.
The prepare step MUST derive all search paths and flags from exports records.
A dependency that needs to alter consumer behavior MUST be modeled as
declared data the consumer interprets, and the interpretation MUST live in
the consumer's build system module.

#### Scenario: Dependency ships only data

- GIVEN an adopted dependency output
- WHEN its consumer builds
- THEN the consumer build MUST execute no code from the dependency output
  during prepare or phase setup.

#### Scenario: Hook-shaped input rejected

- GIVEN a spec field attempting to inject an executable hook from a
  dependency
- WHEN evaluation validates the spec
- THEN Mantle MUST reject the field with a typed error.

### Requirement: Typed build-system registry

r[mantle.dependency_exports.typed_build_systems] Build systems MUST be
registered by name, each bringing its tools, its phase names, and its typed
options.

A spec naming an unregistered build system, an unknown option, or an unknown
phase MUST fail evaluation with the name and the known set. Option types MUST
be checked at evaluation time. Phases MUST be data: a list of registered
phase names or inline named steps, never an opaque string evaluated later.

#### Scenario: Unknown option fails evaluation

- GIVEN a spec using a registered build system with an unknown option key
- WHEN evaluation validates the spec
- THEN evaluation MUST fail naming the option and the known options.

#### Scenario: Wrong option type

- GIVEN a registered option declared as a list and supplied as a string
- WHEN evaluation validates the spec
- THEN evaluation MUST fail with the option name and expected type.

### Requirement: Adoption is gated on a named consumer

r[mantle.dependency_exports.bounded_adoption_gate] Implementation MUST NOT
start until a recorded admission decision names the first consumer, its
target outcome, the adoption path, and the maintenance owner.

The decision MUST be recorded as evidence in this change. Until then the
change remains a proposal-stage design prior and every implementation task
MUST stay open.

#### Scenario: Admission recorded

- GIVEN a written admission decision with consumer, outcome, path, and owner
- WHEN implementation tasks are reviewed
- THEN work MAY start on the named consumer surface only.

#### Scenario: No admission

- GIVEN no admission decision recorded
- WHEN any implementation task is proposed for completion
- THEN the gate MUST treat the claim as blocked.
