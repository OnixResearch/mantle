# Build with Crunch — Tiered Integration Tests

## Why

crunch can compile and run `hello.ncl`, but we haven't exercised the full
surface area end-to-end: fetchers, seed toolchains, multi-output derivations,
inter-derivation dependencies, mkDerivation, self-referential package sets.
Several examples exist (`examples/*.ncl`) but nobody has verified they work
against the current codebase. And the examples themselves stop short of
real-world usage — building actual C programs, linking libraries, fetching
remote sources.

We need a structured progression from "does `echo > $out` work?" through
"can crunch build a C program with dependencies?" so that each tier
exercises a distinct layer of functionality.

## What Changes

- **Tier 1 — Smoke**: Minimal derivations (`/bin/sh`, busybox). No seed,
  no network. Validates the core eval→sandbox→persist pipeline.
- **Tier 2 — Seed toolchain**: Use `crunch bootstrap` + seed.ncl to build
  C programs with bash/coreutils/gcc. Validates closure resolution and
  multi-input sandbox mounts.
- **Tier 3 — Fetchers**: `fetchurl`, `fetchTarball`, `fetchGit` with
  known hashes. Validates the builtin:fetchurl bypass and FOD hash
  verification.
- **Tier 4 — Composition**: Multi-output derivations, inter-package
  dependencies (libfoo→app), mkDerivation with phases, overrideAttrs.
  Validates the dependency DAG, output selection, and the Nickel stdlib.
- **Tier 5 — Real packages**: Build something non-trivial from source
  (jq, a small Rust crate, or similar) using fetched tarballs and the
  seed toolchain. Validates that crunch can replace `nix-build` for
  simple real-world packages.

## Capabilities

### New Capabilities
- `tiered-build-tests`: Structured progression of build complexity
- `real-package-build`: Build actual upstream software from source

### Modified Capabilities
- `examples`: Existing examples become tier targets with verified hashes

## Impact

- **Files**: `examples/*.ncl` (new and updated), `tests/` (integration tests)
- **APIs**: No API changes — this exercises existing functionality
- **Dependencies**: None
- **Testing**: Each tier is a set of `crunch build` invocations with
  expected output paths verified
