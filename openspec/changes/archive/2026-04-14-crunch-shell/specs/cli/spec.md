## ADDED Requirements

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
