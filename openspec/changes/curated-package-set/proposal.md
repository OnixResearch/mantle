## Why

crunch has a working build engine (sandbox, fetchers, CA derivations, dependency ordering) but no packages to build against. Every new project requires hand-authoring Nickel derivations for its full dependency closure — toolchains, C libraries, coreutils, dev tools. This makes crunch unusable for real projects despite the infrastructure being solid.

A curated package set provides the minimum viable ecosystem: the packages one person actually uses, expressed as Nickel derivations that compose with crunch's existing `mkDerivation` and fetcher infrastructure.

## What Changes

- **Bootstrap package set**: A `packages/` directory (or similar) containing Nickel derivation expressions for a curated set of commonly-needed packages. Initial target: ~50-100 packages covering toolchains (Rust, GCC, clang), core utilities (coreutils, findutils, diffutils, grep, sed, gawk), common C libraries (openssl, zlib, pkg-config), and dev tools (git, ripgrep, fd, jq).
- **Package composition model**: A lightweight record-merge or overlay pattern for extending/overriding the base set without forking it. Nickel's merge semantics are the natural fit here — no new abstraction needed.
- **Dependency resolution within the set**: Packages reference each other by name within the set. crunch's existing dependency-ordering in the build pipeline handles the DAG.
- **Bootstrap integration**: The package set builds on top of `bootstrap/seed.ncl` and the existing bootstrap chain. Early packages (make, coreutils, gcc, binutils) are already partially covered by bootstrap derivations — the package set formalizes and extends them.
- **Versioning**: Packages pin specific upstream release tarballs with BLAKE3 hashes. `crunch project refresh` can update these pins over time.

## Capabilities

### New Capabilities
- `package-set`: A checked-in set of Nickel derivations importable by any crunch project
- `package-override`: Merge-based customization of individual packages without forking the set
- `package-search`: CLI lookup of available packages by name (`crunch search` or similar)

### Modified Capabilities
- `build`: No changes to the build engine itself — packages are normal derivations
- `bootstrap`: Bootstrap derivations may be refactored to share structure with the package set

## Impact

- **Files**: New `packages/` directory tree, possibly `packages/lib.ncl` as entry point
- **APIs**: No Rust API changes — packages are pure Nickel
- **Dependencies**: No new Rust deps
- **Testing**: Each package should be buildable individually via `crunch build`. A CI-like `crunch build packages/all.ncl` target exercises the full set.
