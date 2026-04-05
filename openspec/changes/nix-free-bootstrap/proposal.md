# Nix-Free Bootstrap

## Why

crunch proved it can compile C without Nix at runtime (`bootstrap-no-nix.ncl`):
fetch a static musl-gcc, run it in the sandbox with busybox, produce a static
binary. But this is a demo, not a system. Two Nix dependencies remain in the
codebase:

1. **`crunch bootstrap`** shells out to `nix-build` to resolve package names
   to store paths. Used to generate `seed.ncl`.
2. **`nix-store -qR`** resolves runtime closures for seed inputs. Called per
   source input at build time.
3. **`SNIX_BUILD_SANDBOX_SHELL`** is a compile-time path to a static shell.
   Currently points into `/nix/store/`.

The first two are runtime Nix dependencies. The third is a compile-time
constant that happens to reference Nix but doesn't require Nix to be installed
at runtime — the binary is self-contained once baked in.

The `bootstrap-no-nix.ncl` proof showed the architecture already supports
Nix-free operation: static binaries have empty closures, fetched derivations
become `input_derivations` (not `input_sources`), and `resolve_nix_closure`
gracefully returns empty when `nix-store` isn't available. The path is clear.

## What Changes

### Phase 1: Nix-free seed via fetch

Replace `crunch bootstrap --from-nix` (shells out to `nix-build`) with a
fetch-based seed: `crunch bootstrap --fetch` downloads static toolchain
tarballs and writes `seed.ncl` with the output paths.

- Fetch musl-gcc from musl.cc (static, self-contained)
- Fetch busybox-static (for coreutils applets in the sandbox)
- Fetch gnumake (or build it from source with musl-gcc)
- Output: `seed.ncl` referencing crunch store paths, not Nix store paths

### Phase 2: Eliminate nix-store -qR

`resolve_nix_closure()` in `references.rs` calls `nix-store -qR` to get
transitive deps for source inputs. Static binaries don't need this (empty
closure). But dynamically-linked Nix seed packages do.

With a fully static seed (Phase 1), this code path returns empty for every
input. Make this explicit: if all inputs are crunch-built (not from Nix),
skip closure resolution entirely. Keep the Nix fallback for compatibility
with `--from-nix` seeds.

### Phase 3: Build core tools from source

Use the fetched static musl-gcc to build essential tools from source:

- make (gnumake or bmake — needed for most C projects)
- bash (needed as a builder for complex derivations)
- coreutils (or keep using busybox)
- patch, sed, awk, grep, diffutils, findutils

This produces a self-sufficient build environment where every tool was
built by crunch, traced back to the fetched musl-gcc seed.

### Phase 4: Rebuild gcc from source

Use Phase 3 tools to build gcc + binutils + musl from source. This is the
second bootstrap stage: the compiler that compiled itself. After this,
the seed musl-gcc can be discarded.

### Phase 5: Self-host crunch

Build the crunch binary itself using crunch:

- Fetch the Rust toolchain (rustc + cargo) as static binaries
- Or: build rustc from source using the Phase 4 gcc (very ambitious)
- Simpler: fetch a pre-built static rustc, use it to compile crunch

## Capabilities

### New Capabilities
- `fetch-bootstrap`: Generate seed.ncl from fetched static tarballs
- `nix-free-build`: Build software with no Nix binaries on PATH
- `source-bootstrap`: Build core tools from source using the fetched seed

### Modified Capabilities
- `bootstrap`: Add `--fetch` mode alongside existing `--from-nix`
- `closure-resolution`: Skip for static/crunch-built inputs

## Impact

- **Files**: `src/bootstrap.rs`, `crates/crunch-build/src/references.rs`,
  `crates/crunch-build/src/orchestrate.rs`, `examples/`, `lib/`
- **APIs**: `crunch bootstrap` gains `--fetch` flag
- **Dependencies**: None new (ureq already available for fetching)
- **Testing**: Each phase is independently testable via `crunch build`
  with the appropriate seed
