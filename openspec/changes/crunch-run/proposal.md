## Why

There is no complete way to ad-hoc run a package from either a project selector or a standalone Nickel derivation file. `nix run nixpkgs#ripgrep -- rg foo` is one of the most common Nix workflows — build a package, run its binary, throw everything away. crunch needs the same build-and-execute path without involving profiles or dev shells.

A partial `run` command already exists in `src/main.rs` for `crunch.ncl` project packages. The build pipeline and store plumbing are in place. The gap is closing the command contract: direct file targets, explicit binary selection, deterministic fallback selection, argument passthrough, and validation.

## What Changes

- **`crunch run [target] [-- args...]`**: Build a Nickel derivation expression, find a binary in the selected output, and run it with the provided arguments. The derivation is built normally through the pipeline and cached in the store.
- **Target forms**: Support current package-project defaults from `crunch.ncl`, project selectors such as `.#hello`, bare project package names such as `hello`, and explicit `.ncl` file paths that evaluate to exactly one derivation. Selectors and bare project package names are resolved before any filesystem probing; only explicit path syntax (`./`, `../`, `/`, or a `.ncl` suffix) is treated as a file target. This uses the existing package/build project file (`crunch.ncl`), not the dependency-management manifest (`crunch-project.ncl`).
- **Binary discovery**: Support `--bin <name>` to select a specific binary from multi-binary outputs. Without `--bin`, choose the first file or symlink under `$out/bin` sorted by file name.
- **Ephemeral by default**: Built outputs live in the store like any other build. No special cleanup — GC handles it. No profile/generation involvement.
- **Package set integration**: Curated package-set lookup for global bare names remains a later change. This change treats bare names as current-project package selectors.

## Capabilities

### New Capabilities
- `run`: Build and execute a single package in one command
- `run-bin-select`: Choose which binary to run from a multi-output package

### Modified Capabilities
- `build`: No changes — `run` delegates to the existing build pipeline

## Non-Goals

- No curated package-set lookup for global bare package names.
- No `--strict-hermetic` flag on `run`; strict run execution can be added later once build-entry policy explicitly includes `run`.
- No profile or generation mutation.
- No shell sidecar activation; development environments stay under `crunch shell` / `crunch develop`.
- No new build backend or evaluator.

## Impact

- **Files**: `src/main.rs` (run orchestration), `tests/project_build_smoke.rs` or focused CLI tests, README command summary
- **APIs**: New CLI subcommand behavior only
- **Dependencies**: None
- **Testing**: Cover project default, project selector, bare project package name, explicit file target, selector/file collisions, `--bin`, missing and non-executable binaries, argument passthrough, child exit status, build-option forwarding, and README/help documentation
