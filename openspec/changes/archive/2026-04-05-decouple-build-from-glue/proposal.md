## Why

`crunch-build` depends on `crunch-glue`:

```toml
crunch-glue = { path = "../crunch-glue" }
```

The build engine uses `crunch_glue::KnownPaths` in orchestrate.rs,
worker.rs, build_request.rs, and dynamic.rs. The build crate cannot
function without the glue crate.

`KnownPaths` is actually two things jammed into one struct:

1. **Conversion-time tracking**: ATerm hash dedup, HDM cache, cycle
   detection (`in_progress` set), structural identity. Used by
   `convert()` during Nickel→Derivation translation.
2. **Build-time registry**: derivation lookup by drv path, CA output
   resolution (`resolve_output()`), `content_addressed` flag, output
   path queries. Used by the Worker and Builder during builds.

The architecture spec says the glue crate can be reused by tools that
produce derivation JSON from sources other than Nickel. The inverse
should also hold: the build engine should be usable without crunch-glue.
If someone constructs `nix_compat::Derivation` structs from a different
source, they shouldn't need `KnownPaths`.

This upward coupling prevents crunch-build from being an independent
build engine — the goal of "modular like snix."

## What Changes

Split `KnownPaths` into two types:

1. **`ConversionCache`** stays in `crunch-glue` — owns ATerm hash dedup,
   HDM cache, cycle detection. Used only during convert().
2. **`DerivationRegistry`** moves to `crunch-build` (or `crunch-store`) —
   owns drv path→derivation lookup, CA output resolution, build-time
   queries. Accepts pre-computed HDMs and derivations without caring how
   they were produced.

The pipeline layer populates `DerivationRegistry` from `ConversionCache`
output. The build engine never imports crunch-glue.

## Capabilities

### New Capabilities
- `DerivationRegistry`: build-time derivation registry in crunch-build
- Clean crate DAG: crunch-eval → crunch-glue → crunch-pipeline ←
  crunch-build (no crunch-build → crunch-glue edge)

### Modified Capabilities
- `KnownPaths`: split into `ConversionCache` + `DerivationRegistry`
- `Worker::want()`, `Builder::prepare_build()`: take `DerivationRegistry`
  instead of `KnownPaths`

## Impact

- **Files**: modified `crates/crunch-glue/src/known_paths.rs`, new type
  in `crates/crunch-build/`, modified `crates/crunch-build/Cargo.toml`
  (remove crunch-glue dep)
- **APIs**: KnownPaths splits; all callers updated
- **Dependencies**: crunch-build no longer depends on crunch-glue
- **Testing**: build tests no longer need crunch-glue test helpers
