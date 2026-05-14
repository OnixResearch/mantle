# CLI Specification

## Purpose

Defines mantle's project-management command surface and operator-facing CLI
behavior for refresh and stale reporting.
## Requirements
### Requirement: Project-management commands

The CLI MUST provide project-management commands in addition to the existing
engine-oriented commands.

Required commands:
- `mantle init`
- `mantle check`
- `mantle show`
- `mantle refresh`
- `mantle list-stale`
- `mantle upgrade`

These commands operate on the project manifest, lockfile, and generated input
files. They MUST delegate to the project-management layer rather than embed
that logic directly in `src/main.rs`.

#### Scenario: Init scaffolds project files

- GIVEN a directory without mantle project files
- WHEN `mantle init` runs
- THEN it creates `mantle-project.ncl`
- AND it creates `mantle.lock`
- AND it creates or documents the generated `.mantle/` directory layout

#### Scenario: Check validates project state

- GIVEN a project with `mantle-project.ncl`, `mantle.lock`, and `.mantle/inputs.ncl`
- WHEN `mantle check` runs
- THEN it validates the manifest and lockfile
- AND it reports drift or schema errors with a non-zero exit code

#### Scenario: Refresh updates selected inputs

- GIVEN a project with multiple named inputs
- WHEN `mantle refresh foo bar` runs
- THEN only those named inputs are refreshed
- AND `mantle.lock` and `.mantle/inputs.ncl` are rewritten if their resolved
  state changes

#### Scenario: Show renders resolved input state

- GIVEN a valid project manifest and lockfile
- WHEN `mantle show` runs
- THEN it prints a human-readable view of the resolved inputs, including
  frozen state, mirrors, patches, and locked revisions or hashes

#### Scenario: Upgrade migrates project files

- GIVEN a project using an older supported schema version
- WHEN `mantle upgrade` runs
- THEN the project manifest and lockfile are migrated to the current version

### Requirement: Refresh and stale commands report resolver failures distinctly

The CLI MUST distinguish successful refresh/stale results from resolver
failures.

`mantle refresh` MUST report per-input failures and exit non-zero when any
selected input cannot be resolved or hashed, even if other inputs were updated
successfully.

`mantle list-stale` MUST report stale inputs and failed checks separately. It
MUST NOT print `all inputs up to date` when any check failed.

#### Scenario: Partial refresh reports updates and failures together

- GIVEN a manifest with one reachable input and one unreachable input
- WHEN `mantle refresh` runs
- THEN it reports the successful update for the reachable input
- AND it reports the failed resolution for the unreachable input
- AND it exits non-zero
- AND successful lock updates are still written

#### Scenario: Stale check failure is not reported as clean

- GIVEN one input is stale and another input cannot be checked
- WHEN `mantle list-stale` runs
- THEN it reports the stale input
- AND it separately reports the failed check
- AND it exits non-zero
- AND it does not print `all inputs up to date`

### Requirement: README reflects the shipped CLI surface

The top-level README MUST describe the current shipped CLI commands, defaults,
and safety or integrity controls accurately.

At minimum it MUST stay aligned with:
- the default logical store prefix `/mantle/store`
- the `--nix-compat` shorthand for `/nix/store`
- planning and diagnostics entry points such as `mantle doctor` and
  `mantle build --plan`
- the project-management commands (`init`, `check`, `show`, `refresh`,
  `list-stale`, `upgrade`)
- dev-shell entry points (`shell` and `develop`)
- attestation and release-evidence commands (`attest` and `release`)
- signing and trust controls that affect build, self-build, and store
  verification workflows
- strict hermetic selection for build-entry commands
- README-linked focused workflow docs when a supported workflow no longer fits
  cleanly in the top-level summary

#### Scenario: Store-path documentation matches current defaults

- GIVEN a reader follows the README store-path documentation
- WHEN they read about logical store paths and defaults
- THEN it states that derivation hashes use `/mantle/store` by default
- AND it explains `--nix-compat` as the compatibility switch for `/nix/store`
- AND it distinguishes logical `--store-prefix` from physical `--store`

#### Scenario: README command list includes current operator workflows

- GIVEN the current CLI binary
- WHEN the README lists supported operator workflows
- THEN it includes `doctor`, `build --plan`, `shell`, `develop`, `attest`,
  and `release`
- AND it does not present those commands as future work or omit them entirely

#### Scenario: README documents signing and strict hermetic controls

- GIVEN the current CLI binary
- WHEN the README explains cache, verification, and build-entry behavior
- THEN it documents signing-key and trusted-key configuration
- AND it describes unsigned trust as an override rather than the default
- AND it documents `--strict-hermetic` as an opt-in stricter build mode

#### Scenario: README describes attestation-facing build output

- GIVEN the current CLI binary
- WHEN the README explains structured build reporting or attestation workflows
- THEN it says successful build outcomes expose artifact attestation references
- AND it gives the reader a path to the attestation and release workflows
  without requiring them to inspect source code first

#### Scenario: Focused workflow docs stay linked and current

- GIVEN the README moves a longer operator workflow into a focused doc
- WHEN a reader follows that README link
- THEN the focused doc includes the current command entry point for that
  workflow
- AND the README still gives enough context for the reader to find it

#### Scenario: Command coverage can be checked against help output

- GIVEN the current top-level `mantle --help` command list
- WHEN a reviewer compares it with the README command summary and any
  README-linked focused workflow docs
- THEN every shipped top-level operator command family appears in that doc set
- AND no shipped operator family is left undocumented

### Requirement: Attestation command surface

The CLI MUST expose attestation commands as a first-class operator surface.

At minimum the CLI MUST provide commands to:
- show an artifact attestation
- assemble a closure attestation for a root
- verify artifact, closure, or project attestation digests against canonical reconstruction
- diff two native attestations
- render a project attestation view from the manifest, lockfile, and built roots

#### Scenario: Operator inspects artifact attestation

- GIVEN a built store path
- WHEN `mantle attest show <path>` runs
- THEN the CLI renders the native artifact attestation for that logical store path

#### Scenario: Operator verifies closure attestation

- GIVEN a rooted closure attestation already persisted by mantle
- WHEN `mantle attest verify <path>` runs for that root
- THEN mantle recomputes the canonical closure digest
- AND it reports whether the persisted digest matches the recomputed digest

### Requirement: Build reporting includes attestation references

`mantle build` and `mantle --json build` MUST report where the generated
attestations can be found or retrieved.

#### Scenario: JSON build report includes attestation references

- GIVEN a successful `mantle --json build`
- WHEN the JSON report is emitted
- THEN each successful outcome includes a reference to its persisted artifact attestation
- AND the report schema distinguishes those references from ordinary build outputs

### Requirement: Strict hermetic build selection

The CLI MUST let operators request strict hermetic execution for build-entry
commands.

At minimum `mantle build` and `mantle self-build` MUST accept a
`--strict-hermetic` flag and pass that selection unchanged into the pipeline or
self-build orchestration.

#### Scenario: Strict build selects strict profile

- GIVEN a Nickel file that can be built normally
- WHEN `mantle build --strict-hermetic hello.ncl` runs
- THEN the pipeline executes with hermeticity mode `strict`
- AND later strict-mode blockers are treated as build errors instead of warnings

#### Scenario: Self-build selects strict profile

- GIVEN a self-build invocation
- WHEN `mantle self-build --strict-hermetic --store /tmp/store` runs
- THEN the self-build flow records hermeticity mode `strict`
- AND later proof-oriented checks can act on that mode selection

### Requirement: Build reporting exposes hermeticity audit facts

`mantle build` and `mantle --json build` MUST report the selected hermeticity
mode and any hermeticity audit events recorded during the run.

Human-readable output MUST summarize degraded execution facts before reporting a
successful result. JSON output MUST expose the same facts in a stable
machine-readable shape.

#### Scenario: Clean strict run reports no degraded facts

- GIVEN a successful strict build with no degraded conditions
- WHEN `mantle --json build --strict-hermetic hello.ncl` runs
- THEN the JSON report records hermeticity mode `strict`
- AND the hermeticity audit-event list is empty

#### Scenario: Practical run reports degraded facts

- GIVEN a successful practical build that recorded a degraded execution fact
- WHEN `mantle --json build hello.ncl` runs
- THEN the JSON report records hermeticity mode `practical`
- AND the report includes the recorded hermeticity audit event
- AND the human-readable output warns that the run used degraded hermeticity

### Requirement: Shell command surface

The CLI MUST provide a first-class shell command for development
environments.

Required behavior:
- `mantle shell` enters or executes inside a shell environment resolved from
  the compatibility-named `crunch.ncl` package root's `devShells`
- `mantle develop` remains as an alias to the same implementation
- `--command <argv...>` executes a single command in the activated shell
  environment and exits
- `--run <script>` passes a script string to `$SHELL -c` inside the activated
  shell environment
- `--with <path>` prepends additional runtime paths to the activated PATH
- `--no-hook` suppresses shell-hook execution
- `--strict-hooks` makes a failing hook fatal

The shell command MUST read activation metadata from a shell sidecar,
construct the environment, and then execute the requested interactive shell,
command, or run script.

#### Scenario: Sidecar env vars reach command mode

- GIVEN a project whose default dev shell publishes env vars in its shell sidecar
- WHEN `mantle shell --command env` runs in that project
- THEN the command sees those env vars
- AND it sees `CRUNCH_SHELL` pointing at the built shell output path

#### Scenario: Alias dispatches to the same handler

- GIVEN the same project and no special flags
- WHEN `mantle develop --help` is compared with `mantle shell --help`
- THEN `develop` is documented as an alias for `shell`
- AND both commands expose the same development-shell behavior

#### Scenario: Mutually exclusive command selectors are rejected

- GIVEN a shell invocation that passes both `--command` and `--run`
- WHEN the CLI parses the arguments
- THEN the command fails before any build starts
- AND the error explains that the two flags conflict

#### Scenario: Missing sidecar fails clearly

- GIVEN a devShell target whose build output does not contain `.crunch-shell.json`
- WHEN `mantle shell --command true` runs for that target
- THEN the command exits non-zero
- AND the error names `.crunch-shell.json`
- AND the error suggests using `mkShell`

#### Scenario: Runtime path overlays are visible to commands

- GIVEN a `--with <path>` argument pointing at a directory whose `bin/`
  subdirectory contains a tool
- WHEN `mantle shell --command <tool>` runs
- THEN the command finds that tool via the activated PATH
- AND the tool runs successfully

#### Scenario: Failing hook is fatal in strict mode

- GIVEN a shell sidecar hook that exits non-zero
- WHEN `mantle shell --strict-hooks --command true` runs
- THEN `mantle shell` exits with the hook's exit code
- AND the command after the hook does not run

### Requirement: Run command surface

The CLI MUST provide `mantle run` as a build-and-execute command for a single
package output.

`mantle run` MUST support these targets:

- no target: build and run the current compatibility-named `crunch.ncl` package project's default package
- a bare name: build and run that package from the current compatibility-named `crunch.ncl` package project
- `.#name` or `.#category.name`: build and run a package-project selector
- an explicit Nickel file path: build and run that expression when it yields exactly one runnable derivation

For this requirement, `crunch.ncl` is the compatibility-named package/build
project file consumed by `mantle build`, `mantle shell`, `mantle develop`, and
`mantle run`. It is distinct from the `mantle-project.ncl` dependency-management
manifest used by `mantle init`, `mantle refresh`, and related
project-management commands.

Target resolution MUST parse project selectors before filesystem probing, so a
selector-shaped target such as `.#foo.ncl` remains a selector instead of a file
path. A bare name without a path separator and without a `.ncl` suffix MUST
resolve as a current-project package name, even if a same-named filesystem entry
exists in the current directory. A non-selector target that starts with `./`,
`../`, or `/`, or ends in `.ncl`, MUST resolve as an explicit Nickel file path.

If no compatibility-named `crunch.ncl` package project is found for a project
default, bare package name, or project selector, the command MUST fail with an
error that mentions `crunch.ncl`. If a project default package is missing, an
unknown selector or bare package name is requested, an explicit file path does
not exist, or an explicit file does not produce exactly one runnable top-level
derivation, the command MUST fail before launching a package binary and identify
the target that could not be run. Explicit file targets MAY evaluate directly to
one derivation or to a singleton record containing one derivation. Records
containing multiple derivations MUST be rejected for `run`; users must select
one derivation through a project selector or a single-derivation file.

When the selected derivation has multiple outputs, `mantle run` MUST select the
`out` output if present; otherwise it MUST select the first output name in
sorted order. Binary discovery then runs inside that selected output.

#### Scenario: Missing package project fails clearly

- GIVEN the current directory has no compatibility-named `crunch.ncl` package project
- WHEN `mantle run hello` runs
- THEN the command exits non-zero before launching a package binary
- AND the error mentions `crunch.ncl`

#### Scenario: Missing default package fails clearly

- GIVEN a project whose compatibility-named `crunch.ncl` has no runnable default package
- WHEN bare `mantle run` runs
- THEN the command exits non-zero before launching a package binary
- AND the error identifies the missing default package selection

#### Scenario: Unknown package target fails clearly

- GIVEN a project whose compatibility-named `crunch.ncl` does not declare `packages.missing`
- WHEN `mantle run missing` runs
- THEN the command exits non-zero before launching a package binary
- AND the error identifies `missing` as the rejected target

#### Scenario: Nonexistent explicit file fails clearly

- GIVEN `./missing.ncl` does not exist
- WHEN `mantle run ./missing.ncl` runs
- THEN the command exits non-zero before launching a package binary
- AND the error identifies `./missing.ncl` as the rejected target

#### Scenario: Explicit file multi-record is rejected

- GIVEN `many.ncl` evaluates to a record containing multiple derivations
- WHEN `mantle run ./many.ncl` runs
- THEN the command exits non-zero before launching a package binary
- AND the error identifies `./many.ncl` as not runnable by `run`

The command MUST accept the run-scoped build flags `--import-path`, `--jobs`,
`--no-substitute`, `--signing-key`, and `--trust-unsigned`, plus the global
store/state flags `--store`, `--state-dir`, `--store-prefix`, and
`--nix-compat`. These flags MUST feed the normal build pipeline, store,
signing, substitution, and store-prefix settings for the build step.

`mantle run` MUST use the practical hermeticity mode until a later change adds
an explicit `--strict-hermetic` run contract.

The executed child process MUST inherit standard input, standard output,
standard error, environment, and current working directory from `mantle run`.
Arguments after `--` MUST be passed to the child unchanged. `mantle run` MUST
exit with the child's exit status code; if the child terminates without an exit
code, `mantle run` MUST exit non-zero.

`mantle run` MUST NOT create, modify, or switch mantle profile generations.

#### Scenario: Project default package runs

- GIVEN a project whose compatibility-named `crunch.ncl` declares `default.package = "hello"`
- WHEN `mantle run --no-substitute` runs in that project
- THEN mantle builds the `hello` package through the normal build pipeline
- AND executes the selected binary from the built output

#### Scenario: Bare project package name runs

- GIVEN a project whose compatibility-named `crunch.ncl` declares `packages.tool`
- WHEN `mantle run tool` runs in that project
- THEN mantle builds `packages.tool`
- AND executes the selected binary from that output

#### Scenario: Project selector runs

- GIVEN a project whose compatibility-named `crunch.ncl` declares `packages.tool`
- WHEN `mantle run .#tool -- --help` runs in that project
- THEN mantle builds `packages.tool`
- AND passes `--help` unchanged to the selected binary

#### Scenario: Explicit file target runs

- GIVEN a Nickel file that evaluates to one derivation whose output contains
  `bin/tool`
- WHEN `mantle run ./tool.ncl -- --version` runs
- THEN mantle builds `./tool.ncl` through the normal build pipeline
- AND runs `bin/tool` with `--version`

#### Scenario: Same-named path does not shadow bare package

- GIVEN a project whose compatibility-named `crunch.ncl` declares `packages.hello`
- AND the current directory also contains a filesystem entry named `hello`
- WHEN `mantle run hello` runs
- THEN mantle treats `hello` as a project package name
- AND does not treat `hello` as an explicit file target

#### Scenario: Selector suffix does not become file target

- GIVEN a project whose compatibility-named `crunch.ncl` declares a selector target named `foo.ncl`
- AND filesystem probing could find a path-like entry for `foo.ncl`
- WHEN `mantle run .#foo.ncl` runs
- THEN mantle treats `.#foo.ncl` as a selector
- AND does not treat it as an explicit file target

#### Scenario: Multi-output derivation selects out

- GIVEN a selected derivation with outputs named `dev`, `doc`, and `out`
- WHEN `mantle run .#multi` runs
- THEN mantle selects the `out` output before binary discovery

#### Scenario: Child status propagates

- GIVEN a package binary that exits with status `7`
- WHEN `mantle run .#failing` runs
- THEN `mantle run` exits with status `7`

#### Scenario: Child inherits execution context

- GIVEN a package binary that prints an inherited environment variable and its current directory
- WHEN `mantle run .#env-printer` runs with that environment variable set
- THEN the child sees the variable value
- AND the child current directory is the `mantle run` current directory
- AND child stdout/stderr are visible through `mantle run` stdout/stderr

#### Scenario: Build flags are forwarded

- GIVEN a runnable explicit file target that requires an import path and writes to a caller-supplied store
- WHEN `mantle run --store <store> --state-dir <state> --import-path <path> --jobs 1 --no-substitute --signing-key <key> --trust-unsigned ./tool.ncl` runs
- THEN the build uses the supplied import path
- AND the selected output is materialized under `<store>`
- AND state and signing material are read or written under `<state>` and `<key>`
- AND the run does not contact a substituter

#### Scenario: Store prefix flags are forwarded

- GIVEN a runnable explicit file target
- WHEN `mantle --store-prefix /example/store run --store <store> ./tool.ncl` runs
- THEN the build uses `/example/store` as the logical store prefix
- WHEN `mantle --nix-compat run --store <store> ./tool.ncl` runs
- THEN the build uses `/nix/store` as the logical store prefix

#### Scenario: Run does not mutate profiles

- GIVEN a package that builds and runs successfully
- WHEN `mantle run .#tool` completes
- THEN no mantle profile generation is created, removed, or switched

### Requirement: Run binary selection

`mantle run` MUST select the executable under the built output's `bin/`
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
- WHEN `mantle run .#multi --bin beta` runs
- THEN mantle executes `bin/beta`
- AND does not execute `bin/alpha`

#### Scenario: Missing binary fails clearly

- GIVEN a built output containing `bin/alpha`
- WHEN `mantle run .#multi --bin missing` runs
- THEN the command exits non-zero before launching a package binary
- AND the error mentions `bin/missing`

#### Scenario: Path-like binary name is rejected

- GIVEN any runnable package
- WHEN `mantle run .#tool --bin ../tool` runs
- THEN the command exits non-zero before launching a package binary
- AND the error mentions the rejected `--bin` value

#### Scenario: Non-executable binary fails clearly

- GIVEN a built output containing a non-executable regular file `bin/tool`
- WHEN `mantle run .#multi --bin tool` runs
- THEN the command exits non-zero before launching a package binary
- AND the error mentions `bin/tool`

#### Scenario: Default binary selection is sorted

- GIVEN a built output containing executable files `bin/zeta` and `bin/alpha`
- WHEN `mantle run .#multi` runs without `--bin`
- THEN mantle executes `bin/alpha`

#### Scenario: Default binary selection skips non-executables

- GIVEN a built output containing non-executable `bin/aaa` and executable `bin/bbb`
- WHEN `mantle run .#multi` runs without `--bin`
- THEN mantle executes `bin/bbb`

### Requirement: README run command coverage

The top-level README command summary MUST document `mantle run`, including its
project-selector/file-target purpose, `--bin` selection, and `--` argument
passthrough, or link to focused CLI documentation that does so.

#### Scenario: README lists run command

- GIVEN the shipped top-level CLI includes `mantle run`
- WHEN a reader checks the README command summary
- THEN `run` is listed with enough information to build and execute a package

#### Scenario: Help lists run options

- GIVEN the shipped top-level CLI includes `mantle run`
- WHEN `mantle run --help` is displayed
- THEN help text includes target usage, `--bin`, and `--` argument passthrough

