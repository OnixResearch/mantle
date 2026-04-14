# Spec: shell-hook-execution

## Summary

Shell hooks declared in `mkShell` run on entry. They are suppressible
and their execution is delegated to the imperative shell, not the core.

## Requirements

### Requirement: Hook string travels through the sidecar

`mkShell { hook = "echo hello" }` MUST produce a sidecar with
`"hook": "echo hello"`. A `mkShell` without `hook` MUST produce a sidecar
with `"hook": null` or no `hook` field.

#### Scenario: Hook present in sidecar

- GIVEN `mkShell { hook = "echo setup-done" }`
- WHEN the shell derivation is built
- THEN `$out/.crunch-shell.json` contains `"hook": "echo setup-done"`

#### Scenario: No hook declared

- GIVEN `mkShell { env = { X = "1" } }` with no `hook` field
- WHEN the shell derivation is built
- THEN `$out/.crunch-shell.json` has `hook` absent or null

### Requirement: Core returns hook as data, never executes it

`ActivationPlan.hook` MUST be `Option<String>`. The core function MUST NOT
call `std::process::Command` or any exec mechanism for hooks. Hook
execution is the sole responsibility of the imperative shell.

#### Scenario: Core with hook

- GIVEN a sidecar with `hook = "setup.sh"`
- WHEN `compute_activation()` runs
- THEN `plan.hook == Some("setup.sh".into())`
- AND no subprocess was spawned

### Requirement: --no-hook suppresses hook execution

When `--no-hook` is passed, the imperative shell MUST skip hook execution
entirely. The core still returns the hook string in the plan (the core does
not know about CLI flags). Suppression is a shell-side decision.

#### Scenario: Suppress hook

- GIVEN a shell with `hook = "echo hello"`
- WHEN `crunch shell --no-hook --command true` runs
- THEN "hello" does NOT appear in stdout or stderr
- AND exit code is 0

### Requirement: Hook failure is a warning by default

If the hook exits non-zero, the imperative shell MUST print a warning to
stderr and continue to the interactive session or `--command`. The warning
MUST include the hook exit code.

#### Scenario: Failing hook in default mode

- GIVEN a shell with `hook = "exit 42"`
- WHEN `crunch shell --command true` runs
- THEN stderr contains a warning mentioning exit code 42
- AND the `--command` still executes
- AND `crunch shell` exit code is 0 (from `true`)

### Requirement: --strict-hooks makes hook failure fatal

When `--strict-hooks` is passed, a non-zero hook exit MUST cause
`crunch shell` to exit immediately with that exit code, without entering
the session or running `--command`.

#### Scenario: Failing hook in strict mode

- GIVEN a shell with `hook = "exit 42"`
- WHEN `crunch shell --strict-hooks --command true` runs
- THEN `crunch shell` exits with code 42
- AND `true` does NOT execute
