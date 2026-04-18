# Design: flake-compat shim

## Context

Crunch already has a strong project model (`crunch.ncl`) and a growing command
surface (`build`, `run`, `shell`, `check`, `refresh`). What it lacks is an easy
interop story for Nix users who want to try a crunch project without first
installing crunch and manually finding runtime helpers such as `bwrap` and a
static busybox.

A compatibility shim should stay thin. Nix must fetch and launch crunch; it must
not become a second project-definition language that re-encodes `crunch.ncl`.

## Goals / Non-Goals

**Goals**
- generate a deterministic flake shim from a crunch project root
- let `nix run` invoke common crunch workflows through bundled wrappers
- let `nix develop` work without a host-installed crunch binary
- keep `crunch.ncl` and the crunch CLI as the only authoritative behavior

**Non-Goals**
- compile crunch project packages into native Nix derivations
- infer per-package flake outputs from `crunch.ncl`
- generate or update `flake.lock`
- replace the existing repo-development `flake.nix` for the crunch repo itself

## Decisions

### 1. Use delegation, not translation

**Choice:** The generated flake will wrap a bundled crunch binary and forward to
crunch subcommands. It will not translate `crunch.ncl` outputs into Nix package,
check, or shell semantics.

**Rationale:** Nix flake evaluation cannot honestly introspect `crunch.ncl`
without either impure subprocess calls or duplicated project semantics. A thin
wrapper keeps one source of truth and avoids promising `nix build .#pkg` when we
cannot guarantee bwrap-backed crunch builds inside a Nix derivation sandbox.

**Alternative:** Generate static per-package flake outputs from discovered
`packages`, `checks`, and `devShells`. Rejected because those names drift when
`crunch.ncl` changes, and it still would not produce honest native Nix
packages.

**Implementation:** Generated apps call wrapper scripts such as `crunch build
"$@"`, `crunch run "$@"`, `crunch shell "$@"`, and similar project commands.

### 2. Bundle runtime helpers in the shim

**Choice:** Wrapper scripts and the generated tooling shell will bundle the
crunch package plus `bubblewrap`, `git`, and an actual static busybox package,
and will export `SNIX_BUILD_SANDBOX_SHELL` to that busybox using an explicit
absolute store path.

**Rationale:** A shim that still depends on host PATH setup does not solve the
real onboarding problem. Bundling the minimum runtime helpers makes `nix run`
and `nix develop` useful on a Nix machine with no separate crunch install.

**Alternative:** Only put crunch itself in the flake and document the rest.
Rejected because it preserves the exact setup trap the shim is meant to remove.

**Implementation:** Wrapper scripts prepend bundled helper `bin/` directories to
PATH in stable order—bundled crunch, `bubblewrap`, `git`, static busybox, then
inherited PATH—before `exec`-ing crunch. On supported systems they select the
static busybox package through the fixed attr path `pkgs.pkgsStatic.busybox`
and use that package's absolute `bin/busybox` path for
`SNIX_BUILD_SANDBOX_SHELL`. If `pkgs.pkgsStatic.busybox` is unavailable, Nix
 evaluation fails clearly on the supported system; the design MUST NOT fall
 back to `bash-static` or another shell implementation. They
derive the runtime project root from the flake's own `self` source path and
`cd` there before launching any wrapped crunch command, so raw argument
passthrough stays unchanged and runtime behavior does not rely on the caller's
current working directory.

### 3. Expose generic app wrappers plus two shell entry points

**Choice:** The generated flake will expose exactly these generic apps in this
change: `default`, `crunch`, `build`, `run`, `shell`, `check`, and `refresh`,
plus `devShells.default` and `devShells.tooling`.

**Rationale:** Generic wrappers avoid stale per-package output enumeration and
still cover the main workflows a Nix user needs. Setting `apps.default` to the
raw crunch wrapper keeps plain `nix run` useful without inventing hidden build
semantics. A tooling shell is useful when `crunch shell` itself is not the
desired entry point.

**Alternative:** Only expose a default app, or only expose `devShells.default`.
Rejected because both `nix run` and `nix develop` are core flake workflows and
should feel first-class.

**Implementation:** `apps.default` is the same raw passthrough wrapper as
`apps.crunch`. `devShells.default` derives the flake project root from `self`,
`cd`s there, sets the same bundled PATH and absolute `SNIX_BUILD_SANDBOX_SHELL`
as the wrappers, then uses a shell hook that execs the bundled `crunch shell`.
`devShells.tooling` derives that same root, `cd`s there, and leaves the user in
a plain helper shell with the same bundled PATH and sandbox-shell export.

### 4. Generate only `flake.nix`

**Choice:** `crunch flake-compat` writes deterministic `flake.nix` content and
never mutates `flake.lock`.

**Rationale:** Lock generation requires Nix-side network evaluation and would
make a pure renderer command impure and more fragile. Users who want pinning can
run `nix flake lock` themselves after generation.

**Alternative:** Auto-run `nix flake lock` from the command. Rejected because it
adds network side effects and a hard runtime dependency on a configured Nix CLI.

**Implementation:** The command supports a custom crunch input reference. When
no override is provided, the renderer embeds `github:brittonr/crunch` as the
default crunch flake input. On supported systems the generated flake resolves
crunch from `inputs.crunch.packages.${system}.default`. Unsupported-system
branches must not force that attr during evaluation. The rendered file is
otherwise deterministic.

### 5. Command flow is explicit about project discovery and overwrites

**Choice:** The command walks upward from the current working directory until it
finds `crunch.ncl`, writes project-root `flake.nix` by default, supports
`--stdout` and `--output <path>`, treats `--stdout` as incompatible with
`--output` and `--force`, creates missing parent directories for explicit
output paths, and refuses to overwrite an existing target unless `--force` is
present.

**Rationale:** Flake generation is a project-scoped workflow, not a free-form
text renderer. The write path must be predictable, and destructive replacement
must be explicit.

**Alternative:** Always overwrite the target, or require an output path every
time. Rejected because silent replacement is risky and required output paths are
annoying for the normal project-root case.

**Implementation:** Delivery mode changes where bytes go, not which bytes are
rendered. The pure renderer returns one deterministic string; the CLI shell owns
project discovery, overwrite checks, and file I/O. Relative `--output` paths
are resolved from the discovered project root, not the invocation directory,
and missing parent directories are created automatically before the write.
Conflicting `--stdout` flag combinations are rejected with explicit errors
before rendering or file generation starts.

### 6. Linux-only support is explicit

**Choice:** The generated shim supports `x86_64-linux` and `aarch64-linux`
only. On other systems, generated apps and develop entry points fail with a
clear Linux-only message instead of pretending the bundled runtime works.

**Rationale:** The bundled runtime helpers in this change depend on Linux
assumptions (`bubblewrap`, static busybox, crunch build/shell behavior). An
explicit gate is safer than silent partial support.

**Alternative:** Emit the same wrappers on every Nix system and rely on helper
package availability or runtime crashes. Rejected because it hides scope and
produces confusing failures.

**Implementation:** Generated output branches on the target system before any
Linux-only helper package is resolved. Supported systems wire the full helper
set. Unsupported systems instead emit explicit Linux-only failure wrappers and
failure shell hooks that avoid touching Linux-only helper attrs during
evaluation. The failure happens at launch/entry time, with a clear message,
before any delegated crunch workflow claims success.

## Verification strategy

- Renderer unit tests compare exact generated bytes for repeated runs, verify
  `--stdout` and file-write content match, prove `--crunch-input` changes only
  the embedded crunch flake reference, and cover custom crunch-input
  references.
- Renderer tests assert that wrapper scripts export an absolute-store-path
  `SNIX_BUILD_SANDBOX_SHELL` pointing at the fixed static-busybox package,
  reject substitution with another shell package, and prepend helper paths in
  exact order: bundled crunch, `bubblewrap`, `git`, static busybox, then
  inherited PATH.
- Nix-side tests prove supported systems fail clearly if the fixed static-
  busybox attr is unavailable, with no fallback to `bash-static` or another
  shell.
- CLI tests cover project-root discovery, default write mode, `--stdout`,
  `--output <path>`, `--force`, existing-target refusal, explicit
  `--stdout --output` and `--stdout --force` conflicts, root-relative output
  resolution from nested working directories, parent-directory creation, and
  outside-project failure behavior.
- CLI and integration tests assert that generation does not create or mutate
  `flake.lock` and that alternate output writes leave the default
  project-root `flake.nix` untouched unless explicitly targeted.
- Integration tests inspect the generated flake to confirm it exposes only the
  generic wrapper apps (`default`, `crunch`, `build`, `run`, `shell`, `check`,
  `refresh`) plus `devShells.default` and `devShells.tooling`, not translated
  project packages.
- PATH-scrubbed integration tests exercise plain `nix run`, delegated
  `nix run .#build`, `.#refresh -- --help`, and raw `.#crunch` passthrough
  style wrappers and prove they do not require host-installed `crunch`, `git`,
  or `bubblewrap`.
- Runtime integration tests launched from outside the project root prove
  generated wrappers and develop entry points are checkout-relocatable and
  target the flake's own project root at runtime.
- Integration tests exercise both `nix develop` entry points, prove that
  `devShells.default` actually delegates to `crunch shell`, and assert that
  each entry point exposes bundled `crunch`, `bubblewrap`, static busybox, and
  `git` on PATH plus the absolute `SNIX_BUILD_SANDBOX_SHELL` export.
- Integration tests and/or rendered-output inspection cover fixed-subcommand
  forwarding for `build`, `run`, `shell`, `check`, and `refresh`, plus raw
  passthrough for `apps.default` and `apps.crunch`, and prove those two wrappers
  are behaviorally identical.
- Unsupported-system coverage proves generated outputs still evaluate on
  unsupported systems but fail with an explicit Linux-only error at
  launch/entry time.

## Determinism constraints

The renderer must not read timestamps, hostnames, ambient PATH contents, or any
other machine-local state. The only content inputs are the explicit renderer
config values such as the selected crunch input reference.

## Risks / Trade-offs

**Wrapper drift** → Keep one pure renderer and verify exact output in tests.

**Users expect native `nix build` package outputs** → Document that the shim is
command delegation, not package translation, and test that generated apps call
crunch subcommands.

**Runtime helper mismatches across hosts** → Bundle exact helper packages in the
flake and set `SNIX_BUILD_SANDBOX_SHELL` explicitly in wrappers.
