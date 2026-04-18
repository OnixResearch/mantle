# Add flake-compat shim

## Why

Crunch projects currently require a preinstalled `crunch` binary plus a few
runtime helpers before a Nix-first user can do anything useful. That is fine
for core contributors, but it is unnecessary friction for downstream users who
already expect `nix run` and `nix develop` entry points.

The repo itself already ships a flake for developing crunch, but a downstream
`crunch.ncl` project has no standard way to expose the same workflows without
asking users to install crunch first. We need a small compatibility layer that
keeps `crunch.ncl` as the source of truth while letting Nix fetch crunch and
its runtime helpers on demand.

## What Changes

- add a `crunch flake-compat` command that generates a deterministic `flake.nix`
  shim for a crunch project
- define the generated shim as a delegation layer that wraps the bundled crunch
  binary instead of translating `crunch.ncl` into native Nix derivations
- expose generic Nix app entry points (`crunch`, `build`, `run`, `shell`,
  `check`, `refresh`) plus two `nix develop` entry points: a default project
  shell path and a tooling-only shell
- bundle runtime prerequisites needed by those wrappers (`bubblewrap`, static
  busybox, and `git`) so the shim works on supported Linux Nix hosts
  (`x86_64-linux` and `aarch64-linux`) without extra manual PATH setup

## Capabilities

### New Capabilities

- `flake-compat-generation`: crunch can generate a checked-in Nix flake shim
  for a crunch project
- `nix-cli-delegation`: Nix users can run crunch workflows through bundled
  wrappers without installing crunch first
- `bundled-crunch-runtime`: the shim carries the crunch runtime helpers needed
  for common build, shell, and refresh flows

## Impact

- **Files**: new CLI plumbing, renderer code, tests, and generated `flake.nix`
  output for projects that opt in
- **Architecture**: adds a Nix interop layer without changing `crunch.ncl` as
  the canonical project model
- **Dependencies**: no new Rust runtime deps required; generated Nix code uses
  existing flake inputs and Nixpkgs packages
- **Testing**: needs deterministic renderer tests plus CLI/integration coverage
  for generated wrapper behavior

## Verification

A reviewer should expect this change to land with:

- deterministic renderer coverage proving `--stdout` and file-write modes emit
  byte-identical `flake.nix` content for the same input reference, that a
  custom `--crunch-input` override rewrites only the embedded crunch flake
  reference, and that repeated runs with the same override stay byte-identical
- wrapper text tests proving bundled helper PATH setup and an absolute
  `SNIX_BUILD_SANDBOX_SHELL` store path to a static busybox
- PATH-scrubbed integration coverage proving delegated `build`, `refresh`, and
  raw `crunch` passthrough wrappers do not require host-installed `crunch`,
  `git`, or `bubblewrap`, and that plain `nix run` uses the same raw passthrough
  wrapper as `apps.crunch`
- `nix develop` coverage proving `devShells.default` delegates to `crunch
  shell`, still targets the flake project root when launched from outside the
  checkout root, and `devShells.tooling` exposes bundled `crunch`,
  `bubblewrap`, static busybox, and `git` plus an absolute
  `SNIX_BUILD_SANDBOX_SHELL`
- unsupported-system coverage proving unsupported systems fail clearly at app
  launch or develop-entry execution instead of pretending the bundled runtime
  works
- CLI coverage for existing-target refusal, `--force` overwrite, alternate
  output paths, relative `--output` resolution from the discovered project
  root, and the rule that `flake.lock` is not created or mutated

## Non-Goals

- promise this shim on non-Linux Nix hosts in this change
- translate crunch packages into native Nix derivations in this change
- promise `nix build .#package-name` for arbitrary crunch package outputs
- replace `crunch shell` with a Nix-native shell implementation
- make flakes the primary crunch project format
