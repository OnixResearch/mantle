# Bootstrap Seed Chain Specification

## Purpose

Defines requirements for replacing the musl.cc binary seed with a
live-bootstrap-derived chain from hex0 to a modern GCC+musl+binutils toolchain.

## ADDED Requirements

### Requirement: Hex0 seed as trust root

The bootstrap chain MUST start from a hex0 seed binary of at most 512 bytes
for the target architecture, checked into the repository under `bootstrap/seeds/`.
The seed binary MUST be the exact output of assembling the stage0-posix hex0
source for AMD64.

#### Scenario: Seed is present and correctly sized

- GIVEN the repository checkout
- WHEN `bootstrap/seeds/AMD64/hex0-seed` is read
- THEN it is at most 512 bytes and matches the stage0-posix hex0 AMD64 binary

### Requirement: Stage0-posix as crunch derivation

The stage0-posix bootstrap (phases 0-28) MUST run as a single crunch derivation
that takes only the hex0 seed and the stage0-posix source tarball as inputs.
The derivation MUST produce mescc-tools (M1, hex2, kaem, blood-elf), M2-Planet,
and mescc-tools-extra (catm, cp, chmod, mkdir, untar, ungz, unbz2, unxz,
sha256sum).

#### Scenario: Stage0-posix builds from hex0

- GIVEN the hex0 seed and stage0-posix source pinned by hash
- WHEN `crunch build bootstrap/stage0-posix.ncl` runs
- THEN the output contains working M2-Planet, hex2, M1, and kaem binaries

### Requirement: GNU mes from stage0-posix output

GNU mes MUST be built as a crunch derivation using only M2-Planet and
mescc-tools from the stage0-posix output. The mes output MUST include both
the Scheme interpreter and the mes C compiler (`mescc`).

#### Scenario: Mes compiles a C program

- GIVEN the stage0-posix output
- WHEN `crunch build bootstrap/mes.ncl` runs
- THEN mes can compile a trivial C program to a working executable

### Requirement: Tinycc from mes

Tinycc 0.9.26 MUST be built using the mes C compiler. Tinycc 0.9.27 MUST
then be built using tinycc 0.9.26 (self-hosting). The final tinycc output
MUST be capable of building early GCC.

#### Scenario: Tinycc self-hosts

- GIVEN the mes output
- WHEN `crunch build bootstrap/tinycc.ncl` runs
- THEN tinycc 0.9.27 can compile C programs including early GCC prerequisites

### Requirement: GCC version ladder

GCC MUST be built through a version ladder where each version is compiled
by the previous. The minimum chain MUST include: tinycc → gcc-4.0.4 →
gcc-4.7.4 → gcc-10.x (or newer). Supporting tools (make, binutils, musl)
MUST be built at each level as needed by the next GCC version.

#### Scenario: Each GCC version builds from the previous

- GIVEN the tinycc output
- WHEN the GCC chain derivations are built in sequence
- THEN each GCC version produces a working C/C++ compiler

#### Scenario: Modern GCC can build the existing bootstrap chain

- GIVEN the final GCC from the version ladder
- WHEN `bootstrap/selftest.ncl` and `bootstrap/integration-test.ncl` are built
- THEN both tests pass using the from-source toolchain

### Requirement: Normalized seed contract preserved

The final output of the full-source chain MUST expose the same normalized seed
contract as the current `bootstrap/seed.ncl`: target-prefixed binutils in
`bin/`, headers at `<target>/include`, `libgcc_s.so*` under `<target>/lib/`,
and provider metadata in `share/crunch-bootstrap/provider.json`.

#### Scenario: Downstream chain unchanged

- GIVEN the from-source seed output
- WHEN `bootstrap/make.ncl` through `bootstrap/crunch.ncl` are built
- THEN all derivations succeed using the new seed with no modifications

#### Scenario: Self-build produces identical binary

- GIVEN the from-source seed
- WHEN `crunch self-build` runs
- THEN stage1 and stage2 binaries are byte-identical

### Requirement: Source tarballs pinned by hash

Every source tarball fetched during the bootstrap chain MUST be pinned by a
content hash in its `.ncl` file. The hash MUST use the same algorithm as
`crunch.fetchTarball` (currently SHA-256 for Nix compatibility).

#### Scenario: Tampered source detected

- GIVEN a stage's source tarball with a modified byte
- WHEN the derivation is built
- THEN the build fails with a hash mismatch error

### Requirement: Legacy seed as development fast-path

The musl.cc-based seed MUST remain available as an opt-in development
fast-path while the full-source chain is being built out. Once the full chain
passes all validation (selftest, integration-test, self-build), the legacy
seed MAY be removed.

#### Scenario: Developer uses legacy seed

- GIVEN a developer who does not want to wait for the full chain
- WHEN they build with the legacy seed option
- THEN the existing musl.cc tarball path is used and all downstream builds work
