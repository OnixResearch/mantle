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

### Requirement: Repeatable self-hosting proof

The repo MUST provide a repeatable proof that a crunch-built `crunch` binary
can rebuild crunch from the same source tree.

The proof MUST include at least two stages:
- stage0: a checkout-built binary runs `crunch self-build` and produces
  stage1
- stage1: the produced stage1 binary runs `crunch self-build` again and
  produces stage2

The proof MUST record which binary drove each stage and where each resulting
binary landed.

#### Scenario: Stage1 drives stage2

- GIVEN a checkout-built `crunch` binary and a writable proof workspace
- WHEN the self-hosting proof runs
- THEN stage0 produces a working stage1 `bin/crunch`
- AND the proof invokes that stage1 binary for the second stage
- AND stage2 produces a working `bin/crunch`

#### Scenario: Produced binary is executable

- GIVEN a completed self-hosting proof run
- WHEN the proof checks the produced stage2 binary
- THEN `crunch --help` or `crunch --version` succeeds
- AND the proof reports the stage2 binary path

### Requirement: Stage2 must rebuild the final crunch output

The self-hosting proof MUST force a fresh final crunch build for stage2.

The proof MAY reuse previously built bootstrap tools such as `bwrap`,
`busybox`, `rust`, or `gcc`, but it MUST NOT accept a cache hit for the final
`*-crunch` output as evidence of self-hosting.

#### Scenario: Final binary cache hit is rejected as proof

- GIVEN stage1 already wrote a `*-crunch` output in the proof store
- WHEN stage2 starts
- THEN the proof invalidates or removes the prior final `*-crunch` output
- AND stage2 performs a fresh final crunch build
- AND the proof fails if stage2 only reports a final-binary cache hit

### Requirement: Checked-in self-hosting proof entry point

The repo MUST provide one checked-in entry point for running the self-hosting
proof from the repo root.

That entry point MUST set the required build environment explicitly before it
invokes the proof. At minimum it MUST account for the Rust nightly toolchain,
C compiler availability, pkg-config / openssl lookup, and
`SNIX_BUILD_SANDBOX_SHELL`.

The entry point MUST reuse the existing ignored self-hosting proof path instead
of reimplementing the stage0 -> stage1 -> stage2 logic in a second place.

#### Scenario: Contributor runs the checked-in proof entry point

- GIVEN a contributor on Linux in the repo root
- AND the required host tools are installed
- WHEN they run the checked-in proof entry point
- THEN it prepares the required build environment
- AND it invokes the canonical ignored self-hosting proof
- AND the proof output still comes from the existing stage0 -> stage1 -> stage2 test path

#### Scenario: Missing prerequisite fails fast

- GIVEN a contributor is missing a required tool or environment input
- WHEN the checked-in proof entry point starts
- THEN it fails before the long proof build begins
- AND the error names the missing prerequisite

### Requirement: Documented self-hosting workflow

The repo MUST document the checked-in proof entry point as the default way to
run the self-hosting proof locally.

The docs MUST name the exact command, list the required host prerequisites, and
state what the proof demonstrates and what it does not.

#### Scenario: Docs and helper agree

- GIVEN the checked-in proof entry point exists
- WHEN a contributor follows the proof instructions in the repo docs
- THEN they run that same checked-in entry point
- AND they do not need to reconstruct the PATH / environment setup by hand
