# `crunch run` CLI Delta

## ADDED Requirements

### Requirement: Run command surface

The CLI MUST provide `crunch run` as a build-and-execute command for a single
package output.

`crunch run` MUST support these targets:

- no target: build and run the current `crunch.ncl` package project's default package
- a bare name: build and run that package from the current `crunch.ncl` package project
- `.#name` or `.#category.name`: build and run a package-project selector
- an explicit Nickel file path: build and run that single derivation expression

For this requirement, `crunch.ncl` is the package/build project file consumed by
`crunch build`, `crunch shell`, `crunch develop`, and `crunch run`. It is
distinct from the `crunch-project.ncl` dependency-management manifest used by
`crunch init`, `crunch refresh`, and related project-management commands.

Target resolution MUST parse project selectors before filesystem probing, so a
selector-shaped target such as `.#foo.ncl` remains a selector instead of a file
path. A bare name without a path separator and without a `.ncl` suffix MUST
resolve as a current-project package name, even if a same-named filesystem entry
exists in the current directory. A non-selector target that starts with `./`,
`../`, or `/`, or ends in `.ncl`, MUST resolve as an explicit Nickel file path.

If no `crunch.ncl` package project is found for a project default, bare package
name, or project selector, the command MUST fail with an error that mentions
`crunch.ncl`. If a project default package is missing, an unknown selector or
bare package name is requested, an explicit file path does not exist, or an
explicit file does not evaluate to exactly one derivation, the command MUST fail
before launching a package binary and identify the target that could not be run.
Explicit file targets MUST evaluate directly to one derivation. Record-valued
file targets MUST be rejected even when the record contains exactly one
derivation; users must select one derivation through a project selector or a
single-derivation file.

When the selected derivation has multiple outputs, `crunch run` MUST select the
`out` output if present; otherwise it MUST select the first output name in
sorted order. Binary discovery then runs inside that selected output.

#### Scenario: Missing package project fails clearly

- GIVEN the current directory has no `crunch.ncl` package project
- WHEN `crunch run hello` runs
- THEN the command exits non-zero before launching a package binary
- AND the error mentions `crunch.ncl`

#### Scenario: Missing default package fails clearly

- GIVEN a project whose `crunch.ncl` has no runnable default package
- WHEN bare `crunch run` runs
- THEN the command exits non-zero before launching a package binary
- AND the error identifies the missing default package selection

#### Scenario: Unknown package target fails clearly

- GIVEN a project whose `crunch.ncl` does not declare `packages.missing`
- WHEN `crunch run missing` runs
- THEN the command exits non-zero before launching a package binary
- AND the error identifies `missing` as the rejected target

#### Scenario: Nonexistent explicit file fails clearly

- GIVEN `./missing.ncl` does not exist
- WHEN `crunch run ./missing.ncl` runs
- THEN the command exits non-zero before launching a package binary
- AND the error identifies `./missing.ncl` as the rejected target

#### Scenario: Explicit file record is rejected

- GIVEN `many.ncl` evaluates to a record containing one or more derivations
- WHEN `crunch run ./many.ncl` runs
- THEN the command exits non-zero before launching a package binary
- AND the error identifies `./many.ncl` as not runnable by `run`

The command MUST accept the run-scoped build flags `--import-path`, `--jobs`,
`--no-substitute`, `--signing-key`, and `--trust-unsigned`, plus the global
store/state flags `--store`, `--state-dir`, `--store-prefix`, and
`--nix-compat`. These flags MUST feed the normal build pipeline, store,
signing, substitution, and store-prefix settings for the build step.

`crunch run` MUST use the practical hermeticity mode until a later change adds
an explicit `--strict-hermetic` run contract.

The executed child process MUST inherit standard input, standard output,
standard error, environment, and current working directory from `crunch run`.
Arguments after `--` MUST be passed to the child unchanged. `crunch run` MUST
exit with the child's exit status code; if the child terminates without an exit
code, `crunch run` MUST exit non-zero.

`crunch run` MUST NOT create, modify, or switch crunch profile generations.

#### Scenario: Project default package runs

- GIVEN a project whose `crunch.ncl` declares `default.package = "hello"`
- WHEN `crunch run --no-substitute` runs in that project
- THEN crunch builds the `hello` package through the normal build pipeline
- AND executes the selected binary from the built output

#### Scenario: Bare project package name runs

- GIVEN a project whose `crunch.ncl` declares `packages.tool`
- WHEN `crunch run tool` runs in that project
- THEN crunch builds `packages.tool`
- AND executes the selected binary from that output

#### Scenario: Project selector runs

- GIVEN a project whose `crunch.ncl` declares `packages.tool`
- WHEN `crunch run .#tool -- --help` runs in that project
- THEN crunch builds `packages.tool`
- AND passes `--help` unchanged to the selected binary

#### Scenario: Explicit file target runs

- GIVEN a Nickel file that evaluates to one derivation whose output contains
  `bin/tool`
- WHEN `crunch run ./tool.ncl -- --version` runs
- THEN crunch builds `./tool.ncl` through the normal build pipeline
- AND runs `bin/tool` with `--version`

#### Scenario: Same-named path does not shadow bare package

- GIVEN a project whose `crunch.ncl` declares `packages.hello`
- AND the current directory also contains a filesystem entry named `hello`
- WHEN `crunch run hello` runs
- THEN crunch treats `hello` as a project package name
- AND does not treat `hello` as an explicit file target

#### Scenario: Selector suffix does not become file target

- GIVEN a project whose `crunch.ncl` declares a selector target named `foo.ncl`
- AND filesystem probing could find a path-like entry for `foo.ncl`
- WHEN `crunch run .#foo.ncl` runs
- THEN crunch treats `.#foo.ncl` as a selector
- AND does not treat it as an explicit file target

#### Scenario: Multi-output derivation selects out

- GIVEN a selected derivation with outputs named `dev`, `doc`, and `out`
- WHEN `crunch run .#multi` runs
- THEN crunch selects the `out` output before binary discovery

#### Scenario: Child status propagates

- GIVEN a package binary that exits with status `7`
- WHEN `crunch run .#failing` runs
- THEN `crunch run` exits with status `7`

#### Scenario: Child inherits execution context

- GIVEN a package binary that prints an inherited environment variable and its current directory
- WHEN `crunch run .#env-printer` runs with that environment variable set
- THEN the child sees the variable value
- AND the child current directory is the `crunch run` current directory
- AND child stdout/stderr are visible through `crunch run` stdout/stderr

#### Scenario: Build flags are forwarded

- GIVEN a runnable explicit file target that requires an import path and writes to a caller-supplied store
- WHEN `crunch run --store <store> --state-dir <state> --import-path <path> --jobs 1 --no-substitute --signing-key <key> --trust-unsigned ./tool.ncl` runs
- THEN the build uses the supplied import path
- AND the selected output is materialized under `<store>`
- AND state and signing material are read or written under `<state>` and `<key>`
- AND the run does not contact a substituter

#### Scenario: Store prefix flags are forwarded

- GIVEN a runnable explicit file target
- WHEN `crunch --store-prefix /example/store run --store <store> ./tool.ncl` runs
- THEN the build uses `/example/store` as the logical store prefix
- WHEN `crunch --nix-compat run --store <store> ./tool.ncl` runs
- THEN the build uses `/nix/store` as the logical store prefix

#### Scenario: Run does not mutate profiles

- GIVEN a package that builds and runs successfully
- WHEN `crunch run .#tool` completes
- THEN no crunch profile generation is created, removed, or switched

### Requirement: Run binary selection

`crunch run` MUST select the executable under the built output's `bin/`
directory deterministically.

If `--bin <name>` is supplied, `<name>` MUST be one `bin/` entry name. It MUST
NOT be empty, absolute, contain a path separator, or be `.` or `..`. The command
MUST execute exactly `$out/bin/<name>`. If that path is absent, is a directory,
is a dangling symlink, is a symlink to a directory, or is not an executable
file/symlink target, the command MUST fail before launching a child process and
name the rejected binary.

If `--bin` is not supplied, the command MUST ignore directories,
non-executable regular files, dangling symlinks, symlinks to directories, and
symlinks to non-executable targets under `$out/bin`, sort executable file and
valid executable symlink entries by file name, and execute the first candidate.
If there is no `bin/` directory or no candidate executable, the command MUST
fail with a clear error.

#### Scenario: Explicit binary is selected

- GIVEN a built output containing `bin/alpha` and `bin/beta`
- WHEN `crunch run .#multi --bin beta` runs
- THEN crunch executes `bin/beta`
- AND does not execute `bin/alpha`

#### Scenario: Missing binary fails clearly

- GIVEN a built output containing `bin/alpha`
- WHEN `crunch run .#multi --bin missing` runs
- THEN the command exits non-zero before launching a package binary
- AND the error mentions `bin/missing`

#### Scenario: Path-like binary name is rejected

- GIVEN any runnable package
- WHEN `crunch run .#tool --bin ../tool` runs
- THEN the command exits non-zero before launching a package binary
- AND the error mentions the rejected `--bin` value

#### Scenario: Non-executable binary fails clearly

- GIVEN a built output containing a non-executable regular file `bin/tool`
- WHEN `crunch run .#multi --bin tool` runs
- THEN the command exits non-zero before launching a package binary
- AND the error mentions `bin/tool`

#### Scenario: Default binary selection is sorted

- GIVEN a built output containing executable files `bin/zeta` and `bin/alpha`
- WHEN `crunch run .#multi` runs without `--bin`
- THEN crunch executes `bin/alpha`

#### Scenario: Default binary selection skips non-executables

- GIVEN a built output containing non-executable `bin/aaa` and executable `bin/bbb`
- WHEN `crunch run .#multi` runs without `--bin`
- THEN crunch executes `bin/bbb`

### Requirement: README run command coverage

The top-level README command summary MUST document `crunch run`, including its
project-selector/file-target purpose, `--bin` selection, and `--` argument
passthrough, or link to focused CLI documentation that does so.

#### Scenario: README lists run command

- GIVEN the shipped top-level CLI includes `crunch run`
- WHEN a reader checks the README command summary
- THEN `run` is listed with enough information to build and execute a package

#### Scenario: Help lists run options

- GIVEN the shipped top-level CLI includes `crunch run`
- WHEN `crunch run --help` is displayed
- THEN help text includes target usage, `--bin`, and `--` argument passthrough
