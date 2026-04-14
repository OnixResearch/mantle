# Spec: shell-environment-activation

## Summary

Environment variables and PATH entries declared in `mkShell` reach the
user's interactive session and `--command` / `--run` subprocesses through
the sidecar and activation plan.

## Requirements

### Requirement: Sidecar env vars appear in activation

`mkShell { env = { FOO = "bar" } }` MUST produce a sidecar containing
`FOO=bar`. `compute_activation()` MUST include `FOO=bar` in the plan's env
map. The imperative shell MUST set `FOO=bar` in the subprocess environment.

#### Scenario: Single env var round-trip

- GIVEN `mkShell { env = { DATABASE_URL = "postgres://localhost/dev" } }`
- WHEN the shell derivation is built and `crunch shell --command env` runs
- THEN stdout contains `DATABASE_URL=postgres://localhost/dev`

#### Scenario: Multiple env vars

- GIVEN `mkShell { env = { A = "1", B = "2", C = "3" } }`
- WHEN `crunch shell --command env` runs
- THEN stdout contains all three assignments

### Requirement: PATH composition follows explicit ordering

Final PATH MUST be: `[--with bin dirs] ++ [sidecar path_entries] ++
[host PATH entries]`. Duplicates MUST be removed with first-occurrence
wins. The core MUST assert the result is non-empty.

#### Scenario: Sidecar PATH precedes host PATH

- GIVEN a sidecar with `path_entries = ["/crunch/store/...-ripgrep/bin"]`
- AND a host PATH of `/usr/bin:/bin`
- WHEN `compute_activation()` runs
- THEN the plan's PATH is
  `["/crunch/store/...-ripgrep/bin", "/usr/bin", "/bin"]`

#### Scenario: Duplicate removal

- GIVEN a sidecar with `path_entries = ["/usr/bin", "/foo/bin"]`
- AND a host PATH of `/usr/bin:/bar/bin`
- WHEN `compute_activation()` runs
- THEN `/usr/bin` appears exactly once, in the sidecar position

### Requirement: Missing sidecar is a clear error

If `$out/.crunch-shell.json` does not exist after a successful build,
`crunch shell` MUST fail with a message identifying the expected path and
suggesting the shell was not built with `mkShell`.

#### Scenario: Plain derivation used as shell target

- GIVEN a derivation built with `mkDerivation` (not `mkShell`)
- WHEN `crunch shell` targets that output
- THEN exit code is non-zero
- AND stderr mentions `.crunch-shell.json` and `mkShell`

### Requirement: CRUNCH_SHELL env var is always set

The activation plan MUST always include `CRUNCH_SHELL=<output_path>` in
the env map, replacing the current `CRUNCH_DEV_SHELL`. This is not a
protected var — it is always overwritten with the current shell output.

#### Scenario: CRUNCH_SHELL value

- GIVEN a shell derivation with output at `/crunch/store/...-myshell`
- WHEN `crunch shell --command env` runs
- THEN stdout contains `CRUNCH_SHELL=/crunch/store/...-myshell`
