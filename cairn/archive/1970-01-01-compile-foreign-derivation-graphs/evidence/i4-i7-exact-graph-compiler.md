# I4-I7 exact graph compiler evidence

Task-ID: I4, I5, I6, I7
Covers: r[foreign_derivation_import.exact_graph_compilation]
Date: 2026-08-01

## Question

Can Mantle compile each reachable foreign derivation exactly once, after its dependencies, with exact target identities and no prefix-only path substitution?

## Inspected evidence

- `src/foreign_graph_compiler.rs`
- `crates/crunch-glue/src/convert.rs`
- `crates/crunch-glue/src/conversion_cache.rs`
- `crates/crunch-glue/src/error.rs`
- Pueue task `7890`: full `crunch-glue` tests
- Pueue task `7891`: focused foreign import and graph compiler tests
- Pueue task `7892`: final graph compiler tests
- Pueue task `7898`: strict focused Clippy

## Decision

I4 through I7 are complete.

The pure graph compiler validates edges before compilation. It computes a deterministic dependency-first order for all reachable nodes. It rejects cycles, missing nodes, repeated edges, unknown outputs, repeated roots, and partial coverage.

The compiler keeps separate derivation, output, and source maps. It rewrites only known store-object bases and preserves path suffixes. Duplicate mappings, cross-map collisions, unknown objects, and leftover foreign prefixes fail closed.

The new `crunch-glue::resolve_derivation_registration` helper accepts one complete resolved derivation and known parent HDMs. It computes the HDM, target outputs, derivation path, prefix-aware ATerm hash, and pending cache entry without I/O. Existing recursive conversion now uses the same helper.

Positive compiler tests cover a deterministic diamond, a multi-output dependency, a fixed-output dependency, target-prefix sensitivity, and source requirements. Negative tests cover cycles, missing nodes, repeated edges, unknown outputs, duplicate maps, fixed-output name drift, unknown embedded paths, unsupported builtins, and partial coverage.

Exact results:

```text
crunch-glue: 85 passed; 0 failed
foreign graph compiler: 4 passed; 0 failed
```

The focused first-party Clippy commands completed with `-D warnings`. Vendored `snix-castore` emitted one existing `dead_code` warning.

A local secondary reviewer suggested a possible duplicate-path concern without a reproducible case. The compiler rejects duplicate entries before it emits a compiled graph, and the negative duplicate-map test passed.

## Owner

Mantle foreign derivation graph compiler and `crunch-glue` registration core.

## Next action

Implement I8 through I11: the executable plan schema, policy binding, source requirements, native units, and a plan validator.
