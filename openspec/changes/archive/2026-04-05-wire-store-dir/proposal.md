## Why

The `--store` CLI flag exists and `nix-compat` has the underlying
`build_store_path_from_fingerprint_parts_with_store_dir()` and
`to_absolute_path_with_prefix()` APIs, but the pipeline between
them is hardwired to `/nix/store`. `convert()` calls
`to_absolute_path()` (which uses the `STORE_DIR` constant),
`Builder` does the same for filesystem checks, and
`derivation_to_build_request` hardcodes `NIX_STORE=/nix/store`
in its sandbox environment. The `--store /opt/crunch` flag is
accepted and parsed but has no effect on path computation or
sandbox setup.

This matters because a configurable store prefix is a stated goal
(portability spec) and a prerequisite for running crunch without
root access to `/nix/store`. It also matters for testing — unit
tests that check filesystem paths currently require write access
to `/nix/store` or skip themselves.

## What Changes

Thread the store directory string through three layers:

1. **crunch-glue `convert()`** — accept a `store_dir: &str`
   parameter, pass it to `calculate_output_paths`,
   `calculate_derivation_path`, and `to_absolute_path_with_prefix`
   calls. `KnownPaths` stores the store dir and uses it for
   path serialization.

2. **crunch-build `Builder` and `derivation_to_build_request()`** —
   accept a `store_dir` parameter. Use `to_absolute_path_with_prefix`
   for all filesystem existence checks. Set `NIX_STORE` in the
   sandbox env from the parameter instead of the constant. Set
   `inputs_dir` from the parameter.

3. **CLI** — pass `args.store` (already parsed) into `convert()`
   and `Builder::new()`.

No new capabilities. The existing `--store` flag starts working.

## Capabilities

### Modified Capabilities

- `derivation-glue`: `convert()` gains a `store_dir` parameter
- `sandboxed-build`: sandbox env and path resolution use the
  configured store dir

## Impact

- **Files**: `crates/crunch-glue/src/convert.rs`,
  `crates/crunch-glue/src/known_paths.rs`,
  `crates/crunch-build/src/orchestrate.rs`,
  `crates/crunch-build/src/build_request.rs`,
  `src/main.rs`, tests across all three crates
- **APIs**: `convert()` and `Builder::new()` gain a parameter.
  Breaking change to internal crate APIs (no external consumers).
- **Dependencies**: None
- **Testing**: Existing tests that hardcode `/nix/store` paths
  in assertions need updating. Cache test can use a tempdir
  instead of writing to `/nix/store`.
