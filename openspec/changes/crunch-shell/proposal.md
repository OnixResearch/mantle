# crunch shell

## Why

`crunch develop` ignores most of what `mkShell` declares. The Rust side reads
the build output path, prepends `$out/bin` to PATH, and execs `$SHELL`. That
drops every environment variable, every hook, and every composition option on
the floor.

Real projects need more than a PATH entry. A Rust project needs `RUST_LOG`.
A database project needs `PGHOST`. A Python project needs a venv activation
hook. None of these work today.

The problem is structural, not cosmetic: all environment logic lives inside
`exec_shell()` as imperative code tangled with subprocess exec. There is no
pure representation of the activation environment, so there is nothing to test,
compose, or reuse.

## What Changes

- extract environment computation into a pure function that takes structured
  metadata and returns a complete activation plan — no I/O, no subprocess
- carry shell metadata as a machine-readable sidecar written by `mkShell`,
  separate from the derivation contract
- make the exec boundary a one-liner: read sidecar, call core, exec result
- add `--command` / `--run` for scripted non-interactive use
- add `--with` for runtime input composition without editing Nickel source
- support hooks declared in `mkShell` with explicit opt-out

## Capabilities

### New Capabilities

- `pure-shell-environment-core`: environment computation is a pure function
  testable without I/O or subprocess execution
- `shell-environment-activation`: declared env vars, PATH entries, and hooks
  from `mkShell` reach the user session
- `shell-hook-execution`: per-shell setup commands run on entry, suppressible
  via `--no-hook`
- `shell-command-mode`: `--command` / `--run` execute a single command inside
  the shell environment and exit
- `shell-composition`: `--with` layers extra store paths into the activation
  environment without editing Nickel source

## Impact

- **Files**: new `crates/crunch-shell/` (pure environment core), new
  `src/shell_cmd.rs` (imperative exec boundary), `builders/mk_derivation.ncl`
  (sidecar generation in `mkShell`), `src/main.rs` (CLI dispatch)
- **Behavior**: `crunch shell` becomes the primary command; `develop` stays as
  an alias. Environment activation becomes correct by default.
- **Testing**: pure core is unit tested with plain values. Integration tests
  build a shell and assert `--command env` output.

## Non-Goals

- Nix flake interop or `nix develop` compatibility shims
- direnv integration (future; shell env export is the prerequisite)
- container-backed shells (`sandbox = 'oci` or `sandbox = 'wasm`)
- auto-activation on `cd` (direnv territory)
- changing how shell derivations are built or content-addressed
