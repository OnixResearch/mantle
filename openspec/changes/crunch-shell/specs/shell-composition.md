# Spec: shell-composition

## Summary

`--with` layers extra inputs into the shell environment at runtime without
editing Nickel source or changing the shell derivation hash.

## Requirements

### Requirement: --with store path adds to PATH

`crunch shell --with <store-path>` MUST add `<store-path>/bin` to the
activation PATH if that directory exists. If `<store-path>/bin` does not
exist, the path is still accepted (the store path may provide libraries or
data, not binaries) but no PATH entry is added for it.

#### Scenario: Store path with bin/

- GIVEN `/crunch/store/...-ripgrep` exists and has a `bin/` subdirectory
- WHEN `crunch shell --with /crunch/store/...-ripgrep --command which rg`
  runs
- THEN `which rg` succeeds and prints a path under that store entry

#### Scenario: Store path without bin/

- GIVEN `/crunch/store/...-headers` exists but has no `bin/` subdirectory
- WHEN `crunch shell --with /crunch/store/...-headers --command env` runs
- THEN the shell activates without error
- AND PATH does not contain a `...-headers` entry

### Requirement: --with project attribute resolves and builds

`crunch shell --with .#mytool` MUST resolve the project attribute
`mytool`, build it if not cached, and add the output path. Resolution and
building happen in the imperative shell before calling the core. The core
receives only resolved `PathBuf` values.

#### Scenario: Build triggered by --with

- GIVEN a project with `packages.mytool` defined
- WHEN `crunch shell --with .#mytool` runs and `mytool` is not cached
- THEN `mytool` is built
- AND its output `bin/` appears in PATH before the shell's own entries

### Requirement: --with entries appear before sidecar entries in PATH

Multiple `--with` flags accumulate in declaration order. All `--with` bin
dirs MUST appear before sidecar `path_entries` in the final PATH.

#### Scenario: Ordering

- GIVEN `--with /store/A --with /store/B` and sidecar
  `path_entries = ["/store/C/bin"]`
- WHEN `compute_activation()` runs
- THEN PATH order is `[A/bin, B/bin, C/bin, ...host...]`

### Requirement: --with does not affect the derivation hash

`--with` is a runtime-only overlay. The shell derivation's store path and
content hash MUST be identical whether `--with` is used or not.

#### Scenario: Hash stability

- GIVEN the same `mkShell` definition
- WHEN built once without `--with` and once with `--with /store/X`
- THEN both builds produce the same derivation hash and output path

### Requirement: Invalid --with values fail early

A non-existent store path or an unresolvable project attribute MUST cause
`crunch shell` to fail with a clear error before entering the shell
environment. The error MUST name the invalid value.

#### Scenario: Missing store path

- WHEN `crunch shell --with /crunch/store/does-not-exist` runs
- THEN exit code is non-zero
- AND stderr names the missing path

#### Scenario: Missing project attribute

- WHEN `crunch shell --with .#nonexistent` runs in a project without that
  attribute
- THEN exit code is non-zero
- AND stderr names the missing attribute
