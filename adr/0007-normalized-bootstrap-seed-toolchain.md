# ADR 0007: Normalized Bootstrap Seed Toolchain

## Status

Accepted (2026-04-11)

## Context

ADR 0006 introduced `bootstrap/seed.ncl` so the stage0 seed could be
swapped without rewriting the whole bootstrap chain. The next problem was
layout drift.

The raw fetched musl.cc native tarball is usable, but it does not present
the exact contract the bootstrap chain now wants:

- GCC is available as `x86_64-linux-musl-gcc`, but several binutils tools
  are only unprefixed (`ar`, `ld`, `nm`, ...).
- libc and headers live at the top level, while some later bootstrap code
  wants a target sysroot view under `<target>/`.
- libgcc is at `lib/libgcc_s.so*`, while later stages want a stable
  target-specific location.

If each bootstrap derivation compensates for those quirks separately, the
seed abstraction leaks and later provider swaps stay expensive.

## Decision

`bootstrap/seed.ncl` now exposes a normalized seed provider named
`musl-seed-toolchain`.

Implementation:
- fetch raw musl.cc native tarball as `musl-gcc-raw`
- build a thin wrapper derivation that copies the raw tree and adds the
  stable contract the rest of the bootstrap chain consumes

The normalized contract is:
- target-prefixed binutils in `bin/` (`<target>-ar`, `<target>-ld`, ...)
- target sysroot include tree at `<target>/include`
- target sysroot libraries at `<target>/lib`
- `libgcc_s.so*` available under `<target>/lib`

Current raw input still comes from:
- `https://musl.cc/x86_64-linux-musl-native.tgz`
- hash `sha256-XpcI34j9YwAQj7qw4DpvXqT1CX00vHcUQbAk/do46jw=`

## Consequences

- Bootstrap derivations can consistently use `seed.name`, `seed.target`,
  and the normalized filesystem layout instead of knowing raw musl.cc
  quirks.
- A later provider swap can focus on reproducing the normalized contract,
  not on re-editing every bootstrap stage.
- The public stage0 provider visible to the bootstrap chain is now
  `musl-seed-toolchain`, not the raw fetched tarball.
- This change does not yet reduce the trust root to a smaller audited
  source seed. It standardizes the contract around the current seed.

## Alternatives Considered

### Keep using the raw tarball directly

Rejected. The raw layout leaks into every bootstrap stage and defeats the
point of the seed abstraction.

### Switch to the musl.cc cross tarball directly

Rejected for now. The cross tarball changed host-binary behavior and was
not needed once the native tarball could be normalized into the same
public contract.

### Add ad-hoc symlink logic in every bootstrap stage

Rejected. Duplicates seed-layout knowledge across the tree again.
