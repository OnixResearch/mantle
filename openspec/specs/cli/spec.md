# CLI Specification

## Purpose

Defines crunch's project-management command surface and operator-facing CLI
behavior for refresh and stale reporting.
## Requirements
### Requirement: Project-management commands

The CLI MUST provide project-management commands in addition to the existing
engine-oriented commands.

Required commands:
- `crunch init`
- `crunch check`
- `crunch show`
- `crunch refresh`
- `crunch list-stale`
- `crunch upgrade`

These commands operate on the project manifest, lockfile, and generated input
files. They MUST delegate to the project-management layer rather than embed
that logic directly in `src/main.rs`.

#### Scenario: Init scaffolds project files

- GIVEN a directory without crunch project files
- WHEN `crunch init` runs
- THEN it creates `crunch-project.ncl`
- AND it creates `crunch.lock`
- AND it creates or documents the generated `.crunch/` directory layout

#### Scenario: Check validates project state

- GIVEN a project with `crunch-project.ncl`, `crunch.lock`, and `.crunch/inputs.ncl`
- WHEN `crunch check` runs
- THEN it validates the manifest and lockfile
- AND it reports drift or schema errors with a non-zero exit code

#### Scenario: Refresh updates selected inputs

- GIVEN a project with multiple named inputs
- WHEN `crunch refresh foo bar` runs
- THEN only those named inputs are refreshed
- AND `crunch.lock` and `.crunch/inputs.ncl` are rewritten if their resolved
  state changes

#### Scenario: Show renders resolved input state

- GIVEN a valid project manifest and lockfile
- WHEN `crunch show` runs
- THEN it prints a human-readable view of the resolved inputs, including
  frozen state, mirrors, patches, and locked revisions or hashes

#### Scenario: Upgrade migrates project files

- GIVEN a project using an older supported schema version
- WHEN `crunch upgrade` runs
- THEN the project manifest and lockfile are migrated to the current version

### Requirement: Refresh and stale commands report resolver failures distinctly

The CLI MUST distinguish successful refresh/stale results from resolver
failures.

`crunch refresh` MUST report per-input failures and exit non-zero when any
selected input cannot be resolved or hashed, even if other inputs were updated
successfully.

`crunch list-stale` MUST report stale inputs and failed checks separately. It
MUST NOT print `all inputs up to date` when any check failed.

#### Scenario: Partial refresh reports updates and failures together

- GIVEN a manifest with one reachable input and one unreachable input
- WHEN `crunch refresh` runs
- THEN it reports the successful update for the reachable input
- AND it reports the failed resolution for the unreachable input
- AND it exits non-zero
- AND successful lock updates are still written

#### Scenario: Stale check failure is not reported as clean

- GIVEN one input is stale and another input cannot be checked
- WHEN `crunch list-stale` runs
- THEN it reports the stale input
- AND it separately reports the failed check
- AND it exits non-zero
- AND it does not print `all inputs up to date`

### Requirement: README reflects the shipped CLI surface

The top-level README MUST describe the current shipped CLI commands, defaults,
and safety/integrity controls accurately.

At minimum it MUST stay aligned with:
- the default logical store prefix `/crunch/store`
- the `--nix-compat` shorthand for `/nix/store`
- the project-management commands (`init`, `check`, `show`, `refresh`,
  `list-stale`, `upgrade`)
- signing and trust controls that affect build, self-build, and store
  verification workflows

#### Scenario: Store-path documentation matches current defaults

- GIVEN a reader follows the README store-path documentation
- WHEN they read about logical store paths and defaults
- THEN it states that derivation hashes use `/crunch/store` by default
- AND it explains `--nix-compat` as the compatibility switch for `/nix/store`
- AND it distinguishes logical `--store-prefix` from physical `--store`

#### Scenario: README command list includes project-management commands

- GIVEN the current CLI binary
- WHEN the README lists supported commands
- THEN it includes the project-management commands
- AND it does not describe them as future work or omit them entirely

#### Scenario: README documents signing controls as non-default overrides

- GIVEN the current CLI binary
- WHEN the README explains cache and verification behavior
- THEN it documents signing-key and trusted-key configuration
- AND it describes unsigned trust as an override rather than the default

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
- WHEN `crunch attest show <path>` runs
- THEN the CLI renders the native artifact attestation for that logical store path

#### Scenario: Operator verifies closure attestation

- GIVEN a rooted closure attestation already persisted by crunch
- WHEN `crunch attest verify <path>` runs for that root
- THEN crunch recomputes the canonical closure digest
- AND it reports whether the persisted digest matches the recomputed digest

### Requirement: Build reporting includes attestation references

`crunch build` and `crunch --json build` MUST report where the generated
attestations can be found or retrieved.

#### Scenario: JSON build report includes attestation references

- GIVEN a successful `crunch --json build`
- WHEN the JSON report is emitted
- THEN each successful outcome includes a reference to its persisted artifact attestation
- AND the report schema distinguishes those references from ordinary build outputs

### Requirement: Strict hermetic build selection

The CLI MUST let operators request strict hermetic execution for build-entry
commands.

At minimum `crunch build` and `crunch self-build` MUST accept a
`--strict-hermetic` flag and pass that selection unchanged into the pipeline or
self-build orchestration.

#### Scenario: Strict build selects strict profile

- GIVEN a Nickel file that can be built normally
- WHEN `crunch build --strict-hermetic hello.ncl` runs
- THEN the pipeline executes with hermeticity mode `strict`
- AND later strict-mode blockers are treated as build errors instead of warnings

#### Scenario: Self-build selects strict profile

- GIVEN a self-build invocation
- WHEN `crunch self-build --strict-hermetic --store /tmp/store` runs
- THEN the self-build flow records hermeticity mode `strict`
- AND later proof-oriented checks can act on that mode selection

### Requirement: Build reporting exposes hermeticity audit facts

`crunch build` and `crunch --json build` MUST report the selected hermeticity
mode and any hermeticity audit events recorded during the run.

Human-readable output MUST summarize degraded execution facts before reporting a
successful result. JSON output MUST expose the same facts in a stable
machine-readable shape.

#### Scenario: Clean strict run reports no degraded facts

- GIVEN a successful strict build with no degraded conditions
- WHEN `crunch --json build --strict-hermetic hello.ncl` runs
- THEN the JSON report records hermeticity mode `strict`
- AND the hermeticity audit-event list is empty

#### Scenario: Practical run reports degraded facts

- GIVEN a successful practical build that recorded a degraded execution fact
- WHEN `crunch --json build hello.ncl` runs
- THEN the JSON report records hermeticity mode `practical`
- AND the report includes the recorded hermeticity audit event
- AND the human-readable output warns that the run used degraded hermeticity

### Requirement: Shell command surface

The CLI MUST provide a first-class shell command for development
environments.

Required behavior:
- `crunch shell` enters or executes inside a shell environment resolved from
  `crunch.ncl` `devShells`
- `crunch develop` remains as an alias to the same implementation
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
- WHEN `crunch shell --command env` runs in that project
- THEN the command sees those env vars
- AND it sees `CRUNCH_SHELL` pointing at the built shell output path

#### Scenario: Alias dispatches to the same handler

- GIVEN the same project and no special flags
- WHEN `crunch develop --help` is compared with `crunch shell --help`
- THEN `develop` is documented as an alias for `shell`
- AND both commands expose the same development-shell behavior

#### Scenario: Mutually exclusive command selectors are rejected

- GIVEN a shell invocation that passes both `--command` and `--run`
- WHEN the CLI parses the arguments
- THEN the command fails before any build starts
- AND the error explains that the two flags conflict

#### Scenario: Missing sidecar fails clearly

- GIVEN a devShell target whose build output does not contain `.crunch-shell.json`
- WHEN `crunch shell --command true` runs for that target
- THEN the command exits non-zero
- AND the error names `.crunch-shell.json`
- AND the error suggests using `mkShell`

#### Scenario: Runtime path overlays are visible to commands

- GIVEN a `--with <path>` argument pointing at a directory whose `bin/`
  subdirectory contains a tool
- WHEN `crunch shell --command <tool>` runs
- THEN the command finds that tool via the activated PATH
- AND the tool runs successfully

#### Scenario: Failing hook is fatal in strict mode

- GIVEN a shell sidecar hook that exits non-zero
- WHEN `crunch shell --strict-hooks --command true` runs
- THEN `crunch shell` exits with the hook's exit code
- AND the command after the hook does not run

