# Bootstrap Tools Specification

## Purpose

Defines how mantle bootstraps its own sandbox runtime (bwrap) and sandbox
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
- THEN bwrap source is fetched via `mantle.fetchTarball` from a
  pinned release URL with a verified hash

### Requirement: Self-build uses crunch-built tools

The self-build pipeline MUST use the crunch-bootstrapped busybox and bwrap
instead of externally-provided binaries, after the initial bootstrap.

Once step 2 has exported `bwrap` and `busybox` as root outputs, step 3 MUST
bind the final self-build derivation to those exact exported store entries.
It MUST NOT rediscover those tools later by scanning the proof store for the
first matching sibling output.

The self-build pipeline MUST keep using an executable sandbox shell while it
bootstraps the crunch-built busybox and bwrap roots. A host-prerequisite
preflight alone is not enough; the full proof path MUST still get through the
stage0 `busybox.ncl` bootstrap without falling back to an unusable `/bin/sh`
inside bwrap.

#### Scenario: Self-build with mantle tools

- GIVEN `mantle self-build` runs and the bootstrap chain completes
- WHEN the final mantle compilation derivation is built
- THEN the sandbox shell is the crunch-built busybox
- THEN the sandbox runtime is the crunch-built bwrap

#### Scenario: First-ever bootstrap

- GIVEN a bare machine with no prior mantle state
- WHEN `mantle self-build` runs for the first time
- THEN an external bwrap MUST be on PATH (chicken-and-egg)
- THEN after completion, subsequent self-builds use the crunch-built bwrap

#### Scenario: Proof records exact crunch-built tool selection

- GIVEN a proof store that already contains crunch-built `*-bwrap` and
  `*-busybox` outputs from the bootstrap-tool step
- WHEN the stage1 binary runs the second self-build stage
- THEN the proof output records that bwrap source was `crunch-built`
- AND it records the selected busybox path used for
  `SNIX_BUILD_SANDBOX_SHELL`
- AND the stage2 build log reports those same exact crunch-built tool paths

#### Scenario: Final derivation uses exported tool roots directly

- GIVEN step 2 built root outputs `X-bwrap` and `Y-busybox`
- WHEN step 3 generates the final self-build derivation
- THEN the derivation input list includes `/.../X-bwrap` and `/.../Y-busybox`
- AND the shell script uses those exact paths for `bwrap` and `busybox`
- AND the shell script does not scan `$NIX_STORE/*-bwrap` or `$NIX_STORE/*-busybox`

#### Scenario: Stage0 busybox bootstrap keeps a usable shell

- GIVEN `./scripts/prove-self-hosting.sh --check` succeeds on the host
- AND the self-hosting proof starts stage0 from a checkout-built `mantle`
- WHEN stage0 builds `bootstrap/busybox.ncl`
- THEN the sandbox shell path used by bwrap is executable inside the sandbox
- AND the build does not fail with `bwrap: execvp /bin/sh: No such file or directory`

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

- GIVEN `mantle self-build` completes
- WHEN the output store is scanned
- THEN `*-bwrap/bin/bwrap` exists on disk
- AND `*-busybox/bin/busybox` exists on disk
- AND both are executable

### Requirement: SNIX_BUILD_SANDBOX_SHELL accepts crunch-built path

The `SNIX_BUILD_SANDBOX_SHELL` environment variable MUST accept a path
under the mantle store (e.g., `/mantle/store/xxx-busybox/bin/busybox`)
in addition to `/nix/store` paths.

#### Scenario: Sandbox shell from mantle store

- GIVEN `SNIX_BUILD_SANDBOX_SHELL=/mantle/store/xxx-busybox/bin/busybox`
- WHEN a derivation is built
- THEN the sandbox mounts the busybox binary and uses it as `/bin/sh`
