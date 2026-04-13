# ADR 0008: Reduced musl.cc seed provider

## Status

Accepted (2026-04-13)

## Context

ADR 0006 introduced a shared seed boundary and ADR 0007 normalized the public
stage0 contract around `musl-seed-toolchain`. The remaining problem was the
size and audit surface of the current fetched provider.

Before this change, the normalized wrapper still copied the full musl.cc
native tarball into the public seed output. That meant stage0 trusted and
shipped much more than the bootstrap chain actually executed:

- translated message catalogs under `share/locale`
- Fortran frontend payload (`f951`, `x86_64-linux-musl-gfortran`,
  `libgfortran*`)
- coverage tools (`gcov`, `gcov-dump`, `gcov-tool`)
- LTO helpers (`lto1`, `lto-wrapper`, `lto-dump`)
- gold/profile extras (`ld.gold`, `dwp`, `gprof`)

The Rust `crunch bootstrap --fetch` path also duplicated seed metadata instead
of reusing the checked-in provider module.

## Decision

Keep the current raw trust source for now — the pinned musl.cc native tarball —
but reduce the public provider output to the C/C++ bootstrap surface crunch
actually uses.

Implementation:

- `bootstrap/seed.ncl` remains the single source of truth for the fetchable
  seed provider.
- The provider still fetches `https://musl.cc/x86_64-linux-musl-native.tgz`
  with recursive hash `sha256-XpcI34j9YwAQj7qw4DpvXqT1CX00vHcUQbAk/do46jw=`.
- The wrapper now keeps only the directories and binaries needed for the
  current bootstrap chain: compiler drivers, required binutils, headers,
  sysroot content, runtime libraries, and GCC internals.
- The wrapper explicitly drops locale catalogs, Fortran payload, coverage
  helpers, LTO helpers, and gold/profile extras.
- The reduced provider writes
  `share/crunch-bootstrap/provider.json` into its store output so provenance
  and dropped surface area stay inspectable after fetch bootstrap.
- `crunch bootstrap --fetch` now evaluates the shared `bootstrap/seed.ncl`
  module and builds its `toolchain` derivation instead of re-declaring seed
  metadata in Rust.

## Consequences

- Stage0 still trusts one binary tarball from musl.cc, but the public provider
  is smaller and easier to audit than the raw tarball.
- Later bootstrap derivations keep consuming the same normalized contract;
  they do not learn new provider-specific layout details.
- The fetch bootstrap path and self-build path now share one authoritative seed
  definition.
- This is an intermediate trust-reduction step, not the final full-source
  bootstrap root.

## Alternatives Considered

### Keep copying the full raw tarball into the public provider

Rejected. It preserves unnecessary trust surface and leaves provenance harder
to inspect.

### Jump directly to a tiny full-source bootstrap root

Rejected for now. That is still the long-term goal, but it is a larger design
and implementation change than this incremental reduction.

### Re-declare the reduced provider separately in Rust and Nickel

Rejected. That would recreate the metadata drift ADR 0006 was written to avoid.
