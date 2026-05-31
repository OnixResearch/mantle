## ADDED Requirements

### Requirement: Native dynamic plan ABI
The build engine MUST define a native `mantle-plan-v1` ABI as Mantle-owned typed data with schema version, producer metadata, bounded build units, dependencies, outputs, environment entries, declared source inputs, and provenance claims.
ID: build.engine.dynamic.plans.abi


The ABI validator MUST enforce named implementation limits for plan size, unit count, dependency count, output count, environment count, string bytes, and nesting depth. The validator MUST require the nullable fields `producer.goal_hint`, `sources[].nar_blake3`, and `units[].derivation.fixed_output` to be present as either `null` or a valid value. The validator MUST reject unknown schema versions, duplicate unit IDs, duplicate output names, empty required fields, missing required-nullable fields, invalid store-prefix references, undeclared dependency references, absolute host paths outside declared source/store inputs, and any plan whose canonical form cannot be hashed with BLAKE3.

#### Scenario: Valid plan decodes

- GIVEN a `mantle-plan-v1` artifact with one unit, one output, declared dependencies, and bounded environment entries
- WHEN the build engine decodes and validates it
- THEN validation returns typed units ready for registry insertion
- AND returns a canonical BLAKE3 plan digest

#### Scenario: Malformed plan is rejected

- GIVEN a `mantle-plan-v1` artifact with duplicate unit IDs or an undeclared dependency reference
- WHEN the build engine validates it
- THEN validation fails with a typed dynamic-plan error
- AND no units from that artifact are registered

### Requirement: Declared dynamic-plan outputs
A producer derivation MUST explicitly declare which output names may contain native dynamic plans before the build starts.
ID: build.engine.dynamic.plans.declared.outputs


The post-build scanner MUST inspect only those declared outputs for native dynamic plans. A declared output that is missing, not a regular file, exceeds the named dynamic-plan byte limit, or fails validation MUST produce a structured rejection tied to the producer goal and output name.

#### Scenario: Declared output is scanned

- GIVEN a producer declares output `plan` as a native dynamic-plan output
- AND the build result contains a regular file for `plan`
- WHEN the worker handles build completion
- THEN it reads and validates that file as `mantle-plan-v1`

#### Scenario: Undeclared output is ignored

- GIVEN a producer does not declare output `out` as a native dynamic-plan output
- AND `out` contains bytes that look like a plan
- WHEN the worker handles build completion
- THEN it does not decode `out` as a native dynamic plan

### Requirement: Native dynamic plan scheduling
The worker MUST register validated dynamic-plan units as native build goals during the same run, using the existing lazy scheduler without requiring all goals to be known before the first build dispatch.
ID: build.engine.dynamic.plans.scheduler


Registered units MUST inherit the producer's sandbox and trust policy unless the plan narrows those policies. A dynamic plan MUST NOT widen sandbox permissions, substitute trust, store-prefix policy, or host-path access relative to its producer.

#### Scenario: Accepted plan grows graph

- GIVEN a producer build completes with a valid declared dynamic plan
- WHEN the worker validates the plan
- THEN it inserts each unit into the build registry
- AND enqueues reachable units requested by the plan
- AND continues dispatching without restarting evaluation

#### Scenario: Policy widening is rejected

- GIVEN a dynamic plan requests broader host-path or trust policy than its producer
- WHEN the worker validates the plan
- THEN validation rejects the plan
- AND downstream dynamic units from that plan are not scheduled

### Requirement: Native dynamic plan provenance
Build reports and provenance records MUST distinguish native dynamic plans from compatibility `.drv` discovery and MUST record producer goal ID, declared output name, plan artifact path when an output artifact exists, raw artifact digest when bounded bytes were read, canonical plan digest when validation succeeds, accepted unit IDs, rejected artifact reason when present, and scheduler action.
ID: build.engine.dynamic.plans.provenance


The report MUST be deterministic: unit IDs and rejected outputs are sorted in canonical order, and identical accepted plan content produces the same recorded BLAKE3 digest.

#### Scenario: Report records accepted dynamic plan

- GIVEN a producer emits a valid dynamic plan
- WHEN the build report is rendered
- THEN it includes the producer, output name, plan artifact path, raw artifact digest, canonical plan digest, accepted unit IDs, and native mode label

#### Scenario: Report records rejected dynamic plan

- GIVEN a producer emits an invalid declared dynamic plan
- WHEN the build report is rendered
- THEN it includes the producer, output name, plan artifact path when present, rejection reason, and zero accepted unit IDs for that artifact
