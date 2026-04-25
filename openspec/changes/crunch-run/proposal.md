## Why

There is no way to ad-hoc run a package without first writing a project manifest or shell derivation. `nix run nixpkgs#ripgrep -- rg foo` is one of the most common Nix workflows — build a package, run its binary, throw everything away. crunch has no equivalent.

The `cmd_run` stub exists in `src/main.rs` but it's a placeholder. The build pipeline, store, and shell activation are all in place — this is CLI wiring.

## What Changes

- **`crunch run <target> [-- args...]`**: Build a Nickel derivation expression, find the main binary in the output, and exec it with the provided arguments. The derivation is built normally through the pipeline and cached in the store.
- **Binary discovery**: By default, look for `$out/bin/<derivation-name>`. Support `--bin <name>` to select a specific binary from multi-binary outputs.
- **Ephemeral by default**: Built outputs live in the store like any other build. No special cleanup — GC handles it. No profile/generation involvement.
- **Package set integration**: When the curated package set exists, `crunch run <package-name>` resolves against it. Until then, `crunch run ./path/to/derivation.ncl` works with explicit files.

## Capabilities

### New Capabilities
- `run`: Build and execute a single package in one command
- `run-bin-select`: Choose which binary to run from a multi-output package

### Modified Capabilities
- `build`: No changes — `run` delegates to the existing build pipeline

## Impact

- **Files**: `src/main.rs` (wire subcommand), new `src/run_cmd.rs` or extend `src/build_cmd.rs`
- **APIs**: New CLI subcommand only
- **Dependencies**: None
- **Testing**: Test that `crunch run examples/hello.ncl` builds and executes successfully
