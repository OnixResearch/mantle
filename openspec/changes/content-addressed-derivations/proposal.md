## Why

crunch v0 uses input-addressed derivations: the output path is
computed from the derivation's ATerm hash before the build runs.
Two derivations that produce identical outputs but differ in any
input (different compiler version, different build script comment)
get different output paths. This causes unnecessary rebuilds of
downstream consumers — if `libfoo` is rebuilt with a new compiler
but produces the same `.so`, everything depending on `libfoo`
still rebuilds because the path changed.

Content-addressed (CA) derivations compute the output path after
the build, from the actual output content. Identical outputs get
identical paths regardless of how they were built. This means:

- Changing a build script comment doesn't cascade rebuilds
- Compiler upgrades that don't change output don't cascade
- Two independent build paths that converge to the same output
  share a single store path

The defaults spec already says crunch SHOULD implement CA
derivations. The architecture supports it — `BuildService` returns
output nodes, NAR hashing is already done post-build, and
`KnownPaths` can be extended to handle deferred output paths.

## What Changes

- **Output path computation moves post-build.** For CA derivations,
  output paths are not known before the build. The environment
  uses placeholder strings (already supported via `hash_placeholder`)
  that are rewritten after the content hash is computed.

- **Two-phase path resolution in KnownPaths.** Derivations are
  registered with provisional output paths before the build.
  After the build, the actual content-addressed paths replace
  the provisional ones. Downstream derivations that reference
  the output get the final path.

- **Build output rewriting.** After computing the content hash,
  the output must be scanned for references to the provisional
  path and rewritten to the final path. Self-references (the
  output referring to its own path) require special handling.

- **Nickel stdlib gains an addressing_mode field.** Derivations
  can opt into `'input-addressed` (current behavior, for
  compatibility with seed paths) or `'content-addressed`
  (new default).

- **FODs remain unchanged.** Fixed-output derivations are
  already content-addressed by definition — their output path
  is computed from the declared hash. No changes needed.

## Capabilities

### New Capabilities

- `ca-derivations`: content-addressed output path computation
  for non-fixed-output derivations

### Modified Capabilities

- `derivation-glue`: `convert()` handles CA derivation output
  paths (provisional → final)
- `sandboxed-build`: post-build content hashing and path rewriting
- `nickel-stdlib`: `Derivation` contract gains `addressing_mode`

## Impact

- **Files**: `crates/crunch-glue/src/convert.rs`,
  `crates/crunch-glue/src/known_paths.rs`,
  `crates/crunch-build/src/orchestrate.rs`,
  `vendor/nix-compat/src/store_path/utils.rs`,
  `lib/derivation.ncl`, `lib/contracts.ncl`
- **APIs**: `convert()` output changes shape — output paths
  may be `None` until post-build resolution. `BuildOutcome`
  gains final resolved paths.
- **Dependencies**: None new
- **Testing**: New tests for CA path computation, placeholder
  rewriting, self-reference handling, and the identity property
  (identical outputs → identical paths)
