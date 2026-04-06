# Build Engine Decoupling Specification

## Purpose

Defines the boundary between the conversion layer (crunch-glue) and
the build engine (crunch-build), eliminating the upward dependency.

## ADDED Requirements

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

The pipeline crate (or the binary crate, until crunch-pipeline exists)
MUST translate between `ConversionCache` output and
`DerivationRegistry` input:

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
