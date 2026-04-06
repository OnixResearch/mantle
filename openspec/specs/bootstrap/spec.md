# Bootstrap Specification — Delta

## ADDED Requirements

### Requirement: Fetch-based bootstrap

The system MUST support `crunch bootstrap --fetch` which downloads a static
C toolchain from a pinned URL, persists it as a FOD in the crunch store,
and generates `seed.ncl`.

The fetched seed MUST NOT require Nix to be installed. The toolchain MUST
be statically linked so that no closure resolution is needed.

#### Scenario: Bootstrap from fetch on a machine without Nix

- GIVEN a machine with crunch installed but no Nix
- WHEN `crunch bootstrap --fetch --store ~/crunch-store -o seed.ncl` is run
- THEN a seed.ncl is generated with a valid store path to the fetched toolchain
- AND `crunch build hello-world.ncl -I seed.ncl --store ~/crunch-store` succeeds

#### Scenario: Reproducible fetch

- GIVEN the same musl-gcc tarball URL and hash
- WHEN `crunch bootstrap --fetch` is run on two different machines
- THEN both produce the same store path and seed.ncl content

### Requirement: Closure-free inputs

For source inputs that are crunch-built outputs (exist in `--store`, not in
the host's `/nix/store/`), the system MUST skip closure resolution entirely.
Static binaries have no runtime closure.

For Nix-origin source inputs, closure resolution MUST use PathInfo
references (local redb + remote binary cache narinfo), not `nix-store -qR`.

#### Scenario: Build without nix-store on PATH

- GIVEN a seed from `--fetch` (all static, crunch-built)
- AND `nix-store` is not on PATH
- WHEN `crunch build` runs a derivation using the seed
- THEN the build succeeds (crunch-built paths skip closure walk)

#### Scenario: Mixed seed (some Nix, some fetched)

- GIVEN a seed with some paths from Nix and some from `--fetch`
- WHEN `crunch build` runs
- THEN Nix-origin paths get closure resolution via PathInfo/narinfo
- AND fetched paths skip closure resolution

### Requirement: Busybox applet access in sandbox

The sandbox MUST mount the `SNIX_BUILD_SANDBOX_SHELL` binary at both
`/bin/sh` (for shell scripts) and `/bin/busybox` (for applet dispatch).
Build scripts MUST be able to create symlinks to `/bin/busybox` to get
PATH-accessible applets (mkdir, cp, cat, etc.).

#### Scenario: Busybox applets in build script

- GIVEN a derivation with `builder = "/bin/sh"`
- WHEN the build script runs `/bin/busybox mkdir -p /tmp/tools`
- THEN the directory is created
- AND `ln -sf /bin/busybox /tmp/tools/mkdir` creates a working mkdir command

### Requirement: Source-built toolchain

The system MUST support building core tools from fetched source tarballs
using the static musl-gcc seed. These derivations live in a `bootstrap/`
directory as regular `.ncl` files, not special-cased in Rust.

#### Scenario: Build make from source

- GIVEN the fetched musl-gcc seed
- WHEN `crunch build bootstrap/make.ncl` is run
- THEN a working `make` binary is produced in the crunch store
- AND it can be used as an input to subsequent derivations

#### Scenario: Build dash from source

- GIVEN the fetched musl-gcc seed and from-source make
- WHEN `crunch build bootstrap/dash.ncl` is run
- THEN a working POSIX shell is produced (static-pie ELF)

### Requirement: From-source compiler toolchain

The system MUST support building a complete C compiler toolchain from
source: binutils (assembler, linker), musl libc (headers, CRT objects,
libc.a), and GCC (C compiler). Each tool is a `.ncl` derivation that
chains off earlier bootstrap stages.

#### Scenario: Build complete toolchain from source

- GIVEN the bootstrap chain (musl-gcc seed → make → dash)
- WHEN `crunch build bootstrap/gcc.ncl` is run
- THEN GCC, binutils, and musl are all built from source
- AND the from-source GCC can compile C programs

#### Scenario: Self-test with from-source toolchain

- GIVEN from-source gcc, binutils, and musl (no fetched musl-gcc in direct inputs)
- WHEN `crunch build bootstrap/selftest.ncl` is run
- THEN a C test program compiles and passes 1010 assertions
- AND the binary is statically linked against the from-source musl

## MODIFIED Requirements

### Requirement: Seed import mechanism (modified)

The CLI `crunch bootstrap` subcommand MUST support two modes:

- `crunch bootstrap` (default, renamed from `--from-nix`): queries Nix store
  for tool paths. Requires Nix installed.
- `crunch bootstrap --fetch`: downloads static toolchain tarballs. Does NOT
  require Nix installed.

Both modes produce a `seed.ncl` file. The `--fetch` mode SHOULD be
recommended for new installations.
