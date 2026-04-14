# Spec: shell-command-mode

## Summary

`--command` and `--run` allow non-interactive use of the shell environment.

## Requirements

### Requirement: --command execs argv directly

`crunch shell --command <cmd> [args...]` MUST exec the command directly
inside the activation environment. The command is NOT passed through a
shell interpreter. Arguments after `--command` are the argv.

#### Scenario: Command with arguments

- GIVEN a shell with `env.GREETING = "hi"`
- WHEN `crunch shell --command sh -c 'echo $GREETING'` runs
- THEN stdout is `hi`

#### Scenario: Command exit code propagation

- GIVEN any shell
- WHEN `crunch shell --command sh -c 'exit 7'` runs
- THEN `crunch shell` exits with code 7

### Requirement: --run passes script to $SHELL -c

`crunch shell --run <script>` MUST pass the script string to `$SHELL -c`.
Exactly one string argument. The script runs in the activation environment.

#### Scenario: Run mode

- GIVEN a shell with `env.X = "42"`
- WHEN `crunch shell --run 'echo $X'` runs
- THEN stdout is `42`

### Requirement: --command and --run are mutually exclusive

Passing both `--command` and `--run` MUST be a CLI parse error, caught
before any build or env computation.

#### Scenario: Both flags

- WHEN `crunch shell --command foo --run bar` is invoked
- THEN exit code is non-zero
- AND stderr contains an error about conflicting flags

### Requirement: Interactive mode is the default

Without `--command` or `--run`, `crunch shell` MUST start an interactive
session by exec-ing `$SHELL` (or `/bin/sh` if `$SHELL` is unset) inside
the activation environment.

#### Scenario: Default interactive

- GIVEN a shell with `env.FOO = "bar"`
- WHEN `crunch shell` runs with no `--command` or `--run`
- THEN an interactive shell session starts with `FOO=bar` in the
  environment

### Requirement: ExecTarget is decided by the core

The core MUST return an `ExecTarget` enum variant based on the mode:
`Interactive { shell }`, `Command { argv }`, or `Run { shell, script }`.
The imperative shell matches on this enum and calls
`std::process::Command`. The shell does not decide which mode to use.

#### Scenario: Core returns Command variant

- GIVEN CLI args `--command make test`
- WHEN the core computes the activation plan
- THEN `plan.exec_target` is `ExecTarget::Command { argv: ["make", "test"] }`
