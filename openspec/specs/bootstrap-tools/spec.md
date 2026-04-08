# Bootstrap Tools Specification

## Purpose

Defines how crunch bootstraps its own sandbox runtime (bwrap) and sandbox
shell (busybox-static) from source, removing the last external binary
dependencies.

## Requirements

### Requirement: Bootstrap busybox-static from source

The system MUST include a `bootstrap/busybox.ncl` derivation that builds
a statically-linked busybox binary using the existing bootstrap toolchain
(musl-gcc → make → gcc → musl stages).

#### Scenario: Build busybox

- GIVEN the bootstrap chain through gcc + musl is complete
- WHEN `bootstrap/busybox.ncl` is built
- THEN the output contains `bin/busybox` — a static-pie or static ELF
- THEN `busybox --list` exits 0 and lists applets
- THEN `busybox sh -c 'echo ok'` prints `ok`

#### Scenario: Busybox applets include sandbox requirements

- GIVEN the built busybox binary
- WHEN checking available applets
- THEN at minimum: sh, echo, cat, mkdir, cp, chmod, ln, ls, rm, mv,
  sed, grep, awk, tr, head, tail, sort, wc, test, basename, dirname,
  find, xargs, touch, env, install, readlink, printf, id, uname

### Requirement: Bootstrap bwrap from source

The system MUST include a `bootstrap/bwrap.ncl` derivation that builds
bubblewrap from source. The build MUST NOT require meson or python —
it compiles bwrap's C sources directly with gcc.

#### Scenario: Build bwrap without meson

- GIVEN the bootstrap chain through gcc + musl is complete
- WHEN `bootstrap/bwrap.ncl` is built
- THEN the output contains `bin/bwrap` — a statically-linked ELF
- THEN `bwrap --version` exits 0

#### Scenario: Bwrap source is fetched

- GIVEN no local bwrap source
- WHEN the build runs
- THEN bwrap source is fetched via `crunch.fetchTarball` from a
  pinned release URL with a verified hash

### Requirement: Self-build uses crunch-built tools

The self-build pipeline MUST use the crunch-bootstrapped busybox and bwrap
instead of externally-provided binaries, after the initial bootstrap.

#### Scenario: Self-build with crunch tools

- GIVEN `crunch self-build` runs and the bootstrap chain completes
- WHEN the final crunch compilation derivation is built
- THEN the sandbox shell is the crunch-built busybox
- THEN the sandbox runtime is the crunch-built bwrap

#### Scenario: First-ever bootstrap

- GIVEN a bare machine with no prior crunch state
- WHEN `crunch self-build` runs for the first time
- THEN an external bwrap MUST be on PATH (chicken-and-egg)
- THEN after completion, subsequent self-builds use the crunch-built bwrap

#### Scenario: Proof records crunch-built tool selection

- GIVEN a proof store that already contains crunch-built `*-bwrap` and
  `*-busybox` outputs (from a prior self-build's bootstrap-tool step)
- WHEN the stage1 binary runs the second self-build stage
- THEN the proof output records that bwrap source was `crunch-built`
- AND it records the selected busybox path used for
  `SNIX_BUILD_SANDBOX_SHELL`

#### Scenario: First-stage host fallback stays visible

- GIVEN no crunch-built `bwrap` exists on disk yet
- WHEN the first self-build stage runs
- THEN the output records that host fallback was used
- AND the proof workflow treats that fallback as acceptable only for
  the initial bwrap resolve before bootstrap tools are built

### Requirement: Bootstrap tools exported to disk

The self-build pipeline MUST build bwrap and busybox as separate root
derivations so their outputs are exported to the `--store` directory
on disk. Without this step, they would only exist in castore/PathInfo
(intermediate deps are not exported) and subsequent self-build stages
could not find crunch-built tools.

#### Scenario: bwrap and busybox on disk after self-build

- GIVEN `crunch self-build` completes
- WHEN the output store is scanned
- THEN `*-bwrap/bin/bwrap` exists on disk
- AND `*-busybox/bin/busybox` exists on disk
- AND both are executable

### Requirement: SNIX_BUILD_SANDBOX_SHELL accepts crunch-built path

The `SNIX_BUILD_SANDBOX_SHELL` environment variable MUST accept a path
under the crunch store (e.g., `/crunch/store/xxx-busybox/bin/busybox`)
in addition to `/nix/store` paths.

#### Scenario: Sandbox shell from crunch store

- GIVEN `SNIX_BUILD_SANDBOX_SHELL=/crunch/store/xxx-busybox/bin/busybox`
- WHEN a derivation is built
- THEN the sandbox mounts the busybox binary and uses it as `/bin/sh`
