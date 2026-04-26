# Replace musl.cc seed with live-bootstrap-derived chain

## Why

The current bootstrap seed (`bootstrap/seed.ncl`) fetches a ~50 MB prebuilt
GCC+musl+binutils tarball from musl.cc. This is an opaque binary blob -- the
single largest trust assumption in the entire crunch self-build. Everything
downstream (make, dash, binutils, musl, gcc from source, busybox, bwrap, rust,
crunch) is built from source, but the seed that kicks it all off is not.

The bootstrappable.org ecosystem has solved this problem. stage0-posix starts
from a 256-byte hex0 seed that any programmer can audit by hand, and
live-bootstrap extends that chain through mes, tinycc, and multiple GCC versions
until it reaches a modern toolchain. StagEx packages this into reproducible OCI
stages. The build scripts exist and are proven -- the work is translating them
into Nickel derivations that crunch can build.

## What Changes

Replace `bootstrap/seed.ncl` with a chain of Nickel derivations that start from
the hex0 seed and end at a GCC+musl+binutils toolchain equivalent to what the
musl.cc tarball provides today. Each stage is a standalone `bootstrap/*.ncl`
file. The existing chain from `bootstrap/make.ncl` onward connects to the new
seed output unchanged.

**Source of build scripts:** live-bootstrap's `steps/` directory and
stage0-posix's `kaem.run` scripts. We adapt their build logic into crunch
derivations, not reimplement from scratch.

**Chain summary (approximately 20 new derivations):**

1. hex0 seed (256 bytes, checked into repo)
2. stage0-posix (kaem.run: hex0 → mescc-tools + M2-Planet)
3. GNU mes (Scheme interpreter + C compiler)
4. tinycc 0.9.26 (from mes), then tinycc 0.9.27 (self-hosted)
5. gcc-4.0.4 (first GCC buildable by tinycc)
6. Supporting tools at each level (make, sed, patch, etc.)
7. gcc-4.7.4 (intermediate, needed because gcc-4.0 is too old for modern GCC)
8. gcc-10.5.0 or similar modern GCC
9. musl-1.2.x + binutils-2.41 from modern GCC
10. Normalized seed contract (same interface as today's `seed.ncl`)

**Not in scope:** the Rust bootstrap chain (mrustc → rustc version ladder).
`bootstrap/rust.ncl` continues to fetch a prebuilt Rust toolchain. Replacing
that is a separate, larger project (~40 version-step derivations).

## Non-Goals

- Writing a new bootstrap from scratch. We adapt existing proven build scripts.
- Formal verification of the hex0 seed. We trust the stage0-posix audit trail.
- Bootstrapping Rust from source (mrustc chain). Future change.
- Multi-architecture support. x86_64 only for now.
- Switching from GCC to LLVM/Clang as the bootstrap compiler. The
  bootstrappable ecosystem is GCC-centric; LLVM requires GCC to build.

## Capabilities

### New Capabilities

- `bootstrap-hex0-seed`: crunch builds start from an auditable 256-byte seed.
- `bootstrap-stage0-posix`: stage0-posix runs as a crunch derivation.
- `bootstrap-mes`: GNU mes built from M2-Planet output.
- `bootstrap-tinycc`: tinycc built from mes, then self-hosted.
- `bootstrap-gcc-chain`: gcc-4.0 → gcc-4.7 → modern gcc version ladder.
- `bootstrap-full-source-seed`: the normalized seed contract is satisfied by
  a fully source-built toolchain instead of a fetched binary blob.

### Modified Capabilities

- `bootstrap-seed-provider`: `seed.ncl` exposes the same normalized contract
  but is backed by the from-source chain instead of the musl.cc tarball.

### Removed Capabilities

- `bootstrap-musl-cc-fetch`: the musl.cc tarball fetch is removed.

## Impact

- **Files**: ~20 new `bootstrap/*.ncl` files. `bootstrap/seed.ncl` rewritten.
  The hex0 seed binary checked into `bootstrap/seeds/`. Source tarballs for
  each stage pinned with BLAKE3 digests in each `.ncl` file.
- **APIs**: no API changes. The seed contract interface is unchanged.
- **Dependencies**: stage0-posix, mes, tinycc, GCC, musl, binutils source
  tarballs fetched during build. Each pinned by hash.
- **Build time**: full bootstrap from hex0 will be significantly slower than
  fetching a prebuilt tarball. The fetched-seed path should remain available
  as a development fast-path (`--legacy-seed` or similar).
- **Testing**: each stage validated by building a test program with its output.
  Full chain validated by `crunch self-build` producing the same result.

## How to Validate

1. `crunch build bootstrap/stage0-posix.ncl` succeeds and produces mescc-tools.
2. `crunch build bootstrap/mes.ncl` succeeds and mes can compile C.
3. `crunch build bootstrap/tinycc.ncl` succeeds and tinycc can compile C.
4. Each GCC stage builds successfully from the previous.
5. The final normalized seed passes the existing `bootstrap/selftest.ncl` and
   `bootstrap/integration-test.ncl`.
6. `crunch self-build` works with the new seed and produces a byte-identical
   result to the old seed path (same crunch binary).
7. The musl.cc tarball URL is removed from the codebase.
