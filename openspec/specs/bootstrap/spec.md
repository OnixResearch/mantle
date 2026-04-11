# Bootstrap Specification

## Purpose

Defines crunch's bootstrap entry points, trust inventory, bootstrap maturity
labels, from-source bootstrap chain, and checked-in self-hosting proof.

## Requirements

### Requirement: Seed import mechanism

The CLI `crunch bootstrap` subcommand MUST support two modes:

- `crunch bootstrap` resolves package names from an existing Nix installation
  into `/nix/store` paths and writes `seed.ncl`.
- `crunch bootstrap --fetch` downloads pinned seed tarballs, persists them as
  fixed-output derivations, and writes `seed.ncl` without requiring Nix.

The default `crunch bootstrap` path MUST make the Nix dependency explicit.
The `--fetch` path MUST make the fetched seed dependency explicit.

#### Scenario: Contributor generates a Nix-backed seed file

- GIVEN a machine with Nix installed
- WHEN `crunch bootstrap -o seed.ncl bash coreutils gcc gnumake binutils` is run
- THEN `seed.ncl` contains validated `/nix/store` paths for those packages
- AND the generated file makes clear that the paths came from an existing Nix installation

#### Scenario: Contributor generates a fetch-backed seed file

- GIVEN a machine with crunch installed but no Nix
- WHEN `crunch bootstrap --fetch -o seed.ncl` is run
- THEN `seed.ncl` contains validated store paths for the fetched seed packages
- AND the generated file makes clear that the paths came from pinned fetched tarballs

### Requirement: Fetch-based bootstrap

The system MUST support `crunch bootstrap --fetch` which downloads a static
C toolchain from a pinned URL, persists it as a fixed-output derivation in the
crunch store, and generates `seed.ncl`.

The fetched seed MUST NOT require Nix to be installed. The toolchain MUST be
statically linked so that no runtime closure walk is needed for the seed
itself.

#### Scenario: Bootstrap from fetch on a machine without Nix

- GIVEN a machine with crunch installed but no Nix
- WHEN `crunch bootstrap --fetch --store ~/crunch-store -o seed.ncl` is run
- THEN a `seed.ncl` is generated with a valid store path to the fetched toolchain
- AND `crunch build hello-world.ncl -I seed.ncl --store ~/crunch-store` succeeds

#### Scenario: Reproducible fetch identity

- GIVEN the same pinned musl-gcc tarball URL and recursive hash
- WHEN `crunch bootstrap --fetch` is run on two different machines
- THEN both runs compute the same logical output store path
- AND both generated `seed.ncl` files contain the same logical seed path

### Requirement: Explicit bootstrap trust inventory

The repo MUST document the externally trusted inputs and host prerequisites for
all supported bootstrap entry points.

At minimum, the inventory MUST cover:
- `crunch bootstrap`
- `crunch bootstrap --fetch`
- `crunch self-build`
- `./scripts/prove-self-hosting.sh`

For each path, the repo MUST name:
- externally trusted binaries, tarballs, or seeds,
- required host tools,
- what build or proof evidence the repo currently has,
- which stronger bootstrap claim is still not proven.

#### Scenario: Contributor inspects fetch bootstrap trust roots

- GIVEN a contributor wants to understand `crunch bootstrap --fetch`
- WHEN they read the bootstrap docs
- THEN they can see that the flow currently trusts a pinned fetched musl-gcc seed
- AND they can see which host tools are still required around that flow
- AND they can see that this is not yet the same as a full-source bootstrap

#### Scenario: Contributor inspects self-hosting proof trust roots

- GIVEN a contributor wants to understand the checked-in self-hosting proof
- WHEN they read the bootstrap docs
- THEN they can see that stage0 starts from a checkout-built crunch binary
- AND they can see which host prerequisites the proof helper assumes
- AND they can see what the proof demonstrates today

### Requirement: Bootstrap best-practice checklist is explicit

The repo MUST translate the relevant Bootstrappable Builds best practices into
an explicit checklist for crunch.

At minimum, that checklist MUST cover:
- whether crunch has an alternative build path for the build system itself,
- whether bootstrap binary or tarball provenance is clearly labeled,
- whether bootstrap binaries are reproducible from source end-to-end,
- whether bootstrap traceability or self-hosting checks are automated.

Each checklist line MUST be backed by concrete repo evidence and marked with a
current status such as yes, partial, or not yet.

#### Scenario: Contributor asks whether crunch follows bootstrap best practices

- GIVEN a contributor reads the bootstrap docs or spec
- WHEN they ask whether crunch follows the Bootstrappable Builds guidance
- THEN they can find a checklist with concrete statuses
- AND each status cites repo evidence or a named gap
- AND partial compliance is not presented as full compliance

### Requirement: Bootstrap claims stay separated by maturity level

The repo MUST distinguish between bootstrap maturity levels instead of using a
single overloaded label.

At minimum, the docs and bootstrap spec MUST separate:
- seed-assisted bootstrap,
- self-hosting proof,
- full-source bootstrap,
- reproducible or independently reproduced release evidence.

The repo MUST NOT describe a weaker property as if it proved a stronger one.

#### Scenario: Self-hosting proof is not presented as full-source bootstrap

- GIVEN the checked-in stage1 -> stage2 self-hosting proof exists
- WHEN a contributor reads the README or bootstrap spec
- THEN the repo states that the proof demonstrates a working self-rebuild path
- AND it does not claim that crunch is already fully bootstrapped from a tiny audited source seed
- AND it does not claim bit-for-bit reproducible release outputs unless such evidence is present

### Requirement: Bootstrap roadmap names remaining trust-reduction work

The repo MUST track the major milestones and blockers between the current
bootstrap story and a stronger full-source bootstrap story.

That roadmap MUST identify at least:
- remaining fetched or opaque bootstrap seeds,
- remaining host-tool prerequisites,
- proof gaps between self-hosting and reproducible release evidence,
- the next trust-reduction milestones the project intends to pursue.

#### Scenario: Contributor can see what comes after today’s proof

- GIVEN a contributor reads the bootstrap roadmap
- WHEN they compare current proof coverage to future goals
- THEN they can identify which milestones are already complete
- AND they can identify which trust anchors still remain
- AND they can see that reducing those anchors is tracked work rather than implied completion

### Requirement: Closure-free inputs

The system MUST skip closure resolution for source inputs that are
crunch-built outputs in the selected `--store`.

The system MUST resolve Nix-origin source closures through `PathInfo`
references from the local database or remote narinfo, not `nix-store -qR`.

#### Scenario: Build without nix-store on PATH

- GIVEN a seed from `--fetch` where the seed paths are crunch-built static outputs
- AND `nix-store` is not on PATH
- WHEN `crunch build` runs a derivation using that seed
- THEN the build succeeds
- AND the crunch-built paths skip any closure walk through `nix-store`

#### Scenario: Mixed seed inputs

- GIVEN a seed file with some paths imported from Nix and some paths fetched by crunch
- WHEN `crunch build` runs
- THEN Nix-origin paths get closure resolution through `PathInfo` or narinfo data
- AND fetched static paths skip closure resolution

### Requirement: Busybox applet access in sandbox

The sandbox MUST mount the `SNIX_BUILD_SANDBOX_SHELL` binary at both
`/bin/sh` and `/bin/busybox`.

Build scripts MUST be able to create symlinks to `/bin/busybox` to expose
busybox applets such as `mkdir`, `cp`, and `cat` on `PATH`.

#### Scenario: Busybox applets in build script

- GIVEN a derivation with `builder = "/bin/sh"`
- WHEN the build script runs `/bin/busybox mkdir -p /tmp/tools`
- THEN the directory is created
- AND `ln -sf /bin/busybox /tmp/tools/mkdir` creates a working `mkdir` command

### Requirement: Source-built toolchain

The system MUST support building core tools from fetched source tarballs using
the static musl-gcc seed.

These derivations MUST live in the `bootstrap/` directory as regular `.ncl`
files, not as Rust-only special cases.

#### Scenario: Build make from source

- GIVEN the fetched musl-gcc seed
- WHEN `crunch build bootstrap/make.ncl` is run
- THEN a working `make` binary is produced in the crunch store
- AND it can be used as an input to subsequent derivations

#### Scenario: Build dash from source

- GIVEN the fetched musl-gcc seed and from-source `make`
- WHEN `crunch build bootstrap/dash.ncl` is run
- THEN a working POSIX shell is produced
- AND the output is usable by later bootstrap stages

### Requirement: From-source compiler toolchain

The system MUST support building a complete C compiler toolchain from source:
`binutils`, `musl`, and `gcc`.

Each tool MUST be represented as a `.ncl` derivation that chains off earlier
bootstrap stages.

#### Scenario: Build complete toolchain from source

- GIVEN the bootstrap chain `musl-gcc -> make -> dash`
- WHEN `crunch build bootstrap/gcc.ncl` is run
- THEN `gcc`, `binutils`, and `musl` are all built from source
- AND the from-source `gcc` can compile C programs

#### Scenario: Self-test with from-source toolchain

- GIVEN from-source `gcc`, `binutils`, and `musl` with no fetched musl-gcc in direct inputs
- WHEN `crunch build bootstrap/selftest.ncl` is run
- THEN a C test program compiles and passes its bootstrap self-test assertions
- AND the binary is statically linked against the from-source `musl`

### Requirement: Repeatable self-hosting proof

The repo MUST provide a repeatable proof that a crunch-built `crunch` binary
can rebuild crunch from the same staged source tree.

The proof MUST include at least two stages:
- stage0: a checkout-built binary runs `crunch self-build` and produces stage1
- stage1: the produced stage1 binary runs `crunch self-build` again and produces stage2

The proof MUST record which binary drove each stage, which staged source tree
was used, and where each resulting binary landed.

#### Scenario: Stage1 drives stage2 from the recorded staged source

- GIVEN a checkout-built `crunch` binary and a writable proof workspace
- WHEN the self-hosting proof runs
- THEN stage0 produces a working stage1 `bin/crunch`
- AND stage0 records the staged `*-crunch-src` path it used
- AND the proof invokes that stage1 binary for the second stage
- AND stage2 reuses that same staged source path instead of restaging from the checkout
- AND stage2 produces a working `bin/crunch`

#### Scenario: Stage2 is isolated from checkout staging fallback

- GIVEN the proof has already recorded a staged `*-crunch-src` path from stage0
- WHEN the stage1 binary runs the second stage
- THEN the proof launches stage2 from outside the repo root
- AND it clears `PATH` before the stage2 command starts
- AND the stage2 run still succeeds by reusing the recorded staged source path

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
C compiler availability, pkg-config or openssl lookup, and
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
- AND they do not need to reconstruct the `PATH` or environment setup by hand
