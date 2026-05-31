# Build Engine Decoupling Specification

## Purpose

Defines the boundary between the conversion layer (crunch-glue) and
the build engine (crunch-build), eliminating the upward dependency.
## Requirements
### Requirement: DerivationRegistry in crunch-build

`crunch-build` MUST define a `DerivationRegistry` struct that provides
build-time derivation lookup. It MUST NOT import crunch-glue.

```rust
pub struct DerivationRegistry {
    /// drv absolute path → RegistryEntry
    entries: HashMap<String, RegistryEntry>,
    /// Store directory prefix.
    store_dir: String,
}

pub struct RegistryEntry {
    pub drv_path: StorePath<String>,
    pub hash_derivation_modulo: [u8; 32],
    pub derivation: Derivation,
    pub content_addressed: bool,
    pub resolved_outputs: HashMap<String, StorePath<String>>,
}
```

#### Scenario: Worker uses DerivationRegistry

- GIVEN a Worker processing goals
- WHEN it needs to look up a derivation by drv path
- THEN it calls `registry.get(drv_abs_path)` on DerivationRegistry

#### Scenario: Registry populated from conversion output

- GIVEN crunch-glue's `convert()` produced a derivation and HDM
- WHEN the pipeline adds it to the DerivationRegistry
- THEN the build engine can find it without importing crunch-glue

### Requirement: ConversionCache stays in crunch-glue

`crunch-glue` MUST retain conversion-time tracking in a
`ConversionCache` (renamed from `KnownPaths`):

- ATerm hash dedup
- HDM cache
- Cycle detection (`in_progress` set)
- Structural identity checks

`ConversionCache` MUST NOT contain `resolve_output()`,
`content_addressed`, or any build-time state.

#### Scenario: Convert uses ConversionCache

- GIVEN a CrunchDerivation with diamond dependencies
- WHEN `convert(drv, &mut cache)` is called
- THEN shared deps are converted once (dedup via ATerm hash)
- AND the cache does not track output resolution

### Requirement: crunch-build has no crunch-glue dependency

`crunch-build/Cargo.toml` MUST NOT list `crunch-glue` as a dependency.
The build engine depends only on `nix-compat` (for Derivation struct)
and `snix-*` crates (for store/build traits).

#### Scenario: Independent compilation

- GIVEN the crunch-build crate
- WHEN `cargo build -p crunch-build` is run
- THEN it compiles without crunch-glue in its dependency tree

### Requirement: Pipeline bridges the gap

The pipeline crate (or the binary crate, until crunch-pipeline exists) MUST
translate between `ConversionCache` output and `DerivationRegistry` input:

```rust
fn populate_registry(
    cache: &ConversionCache,
    registry: &mut DerivationRegistry,
) { ... }
```

#### Scenario: End-to-end flow

- GIVEN a .ncl file with nested derivations
- WHEN the pipeline evaluates, converts, and builds
- THEN crunch-glue populates ConversionCache
- AND the pipeline populates DerivationRegistry from it
- AND crunch-build runs the Worker against the registry

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
