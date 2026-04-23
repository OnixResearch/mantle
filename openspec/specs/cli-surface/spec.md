# cli-surface Specification

## Purpose

This spec defines the public `crunch system eval` and `crunch system build`
CLI contract for the native system-config pipeline.
## Requirements
### Requirement: CLI-1 System eval command

The CLI MUST provide `crunch system eval <inventory.ncl>` as the dry-run entry
point for the system-config pipeline.
ID: systemconfig.cli.surface.cli1

The command MUST accept `--stop-after` with exactly two valid values:
`fragments` and `derivations`. The default MUST be `derivations`. Any other
value MUST fail with a CLI diagnostic before evaluation begins.

The command MUST evaluate the selected modules and inventory, stop at the phase
selected by `--stop-after`, and write a `SystemPipelineResult` envelope to
stdout for every non-fatal execution. That stdout envelope MUST preserve the
shared `machines`, `errors`, and `warnings` collections defined by ERR-4, and
it MUST be serialized according to the eval `--format` flag defined by CLI-6.
Successful machine outcomes contain either merged fragment trees
(`--stop-after=fragments`) or dry-run derivation records
(`--stop-after=derivations`, the default). Partial success MUST still emit the
successful machine outcomes plus the collected `errors`/`warnings` on stdout
while rendering the same diagnostics on stderr for operators. Eval stderr MUST
follow ERR-3 whenever the global `--json` flag is active and MUST remain human-
readable otherwise; that rule applies to normal success, partial success, and
all fatal pre-machine failures. If any fatal pre-machine failure occurs before
machine work begins — including CLI argument errors, inventory validation
failures, or loader startup failures such as an unreadable modules directory —
stdout MUST remain empty, stderr MUST carry the fatal diagnostic, and the
command MUST exit non-zero. Otherwise the command MUST exit with status 0 when
no error diagnostics were produced for selected machines, and non-zero when any
error diagnostic was produced.

When `--stop-after=fragments` is selected, eval MUST stop before assembler
resolution or validation begins. In that mode `--assembler <name>` MUST be
accepted for CLI consistency but MUST have no effect, and unknown backend names
MUST NOT produce assembler diagnostics because no assembler lookup occurs.
Backend resolution and unknown-backend diagnostics apply only to
`--stop-after=derivations`, where dry-run assembly is actually requested.

#### Scenario: Eval returns dry-run derivations with partial success preserved
ID: systemconfig.cli.surface.cli1.scenario

- GIVEN an inventory with two selected machines
- AND one machine evaluates successfully to dry-run derivations
- AND the other machine produces a non-fatal module error
- WHEN `crunch system eval <inventory.ncl>` runs
- THEN stdout contains a `SystemPipelineResult` with the successful machine
  outcome, `MachineOutcome::Failed` for the failed selected machine, and the
  collected `errors` and `warnings`
- AND stderr reports the failed machine diagnostic
- AND the command exits non-zero

### Requirement: CLI-2 System build command

The CLI MUST provide `crunch system build <inventory.ncl>` as the build entry
point for the system-config pipeline.
ID: systemconfig.cli.surface.cli2

The command MUST evaluate the selected modules, assemble derivations, and build
successful machine plans through `crunch-pipeline::build()`. It MUST inherit
crunch's existing global `--json` flag. Without `--json`, stdout MUST be a
human-readable summary derived from the in-memory `SystemPipelineResult` that
names successful machines, failed selected machines, and total warning/error
counts, while stderr remains human-readable diagnostics. With
`--json`, stdout MUST be a `SystemPipelineResult` envelope whose `machines`,
`errors`, and `warnings` collections preserve the shared ERR-4 shape, and
whose successful machine outcomes embed the per-machine `reports` array of one
or more `crunch-build-report-v1` payloads produced by
`crunch-pipeline::build()`. Stderr MUST contain only
structured JSON diagnostics as defined by ERR-3. If any fatal pre-machine
failure occurs before machine work begins — including CLI argument errors,
inventory validation failures, or loader startup failures such as an unreadable
modules directory — stdout MUST remain empty, stderr MUST carry the fatal
diagnostic, and the command MUST exit non-zero. Otherwise the command MUST exit
with status 0 only when no error diagnostics were produced.

#### Scenario: Build summary distinguishes mixed-success runs
ID: systemconfig.cli.surface.cli2.scenario

- GIVEN a selected-machine build where one machine succeeds and one machine
  fails
- WHEN `crunch system build <inventory.ncl>` runs without `--json`
- THEN stdout summarizes the successful machine, the failed machine, and
  warning/error counts
- AND stderr carries the human-readable diagnostics for the failed machine

### Requirement: CLI-3 Module directory discovery

Both system commands MUST accept `--modules <dir>` to specify the directory
containing module files.
ID: systemconfig.cli.surface.cli3

When `--modules` is omitted, the default MUST be `./modules/` relative to the
inventory file's parent directory.

#### Scenario: Commands derive the default modules directory from the inventory
ID: systemconfig.cli.surface.cli3.scenario

- GIVEN an inventory at `examples/system-config/inventory.ncl`
- AND no `--modules` flag
- WHEN `crunch system eval` or `crunch system build` runs
- THEN the module directory defaults to `examples/system-config/modules/`

### Requirement: CLI-4 Machine filter

Both system commands MUST accept repeatable `--machine <name>` flags to restrict
evaluation and building to the named machines.
ID: systemconfig.cli.surface.cli4

Machine filtering MUST happen before backend override resolution, before any
evaluator graph construction that would create provider edges or cross-
reference diagnostics, and before any assembler work begins. Diagnostics for
unselected machines MUST NOT be emitted. If any requested machine name is not
present in the inventory, the command MUST fail before evaluation begins, MUST
leave stdout empty, and MUST emit a CLI diagnostic naming the unknown machine.

#### Scenario: Machine filter narrows the selected machine set
ID: systemconfig.cli.surface.cli4.scenario

- GIVEN an inventory with machines `server1` and `server2`
- WHEN `crunch system eval <inventory.ncl> --machine server1` runs
- THEN only `server1` is evaluated
- AND stdout contains no machine outcome for `server2`
- AND requesting `--machine server3` instead would fail before evaluation with
  an empty stdout stream and a CLI diagnostic naming `server3`

### Requirement: CLI-5 Assembler selection precedence

Both `crunch system eval` and `crunch system build` MUST accept
`--assembler <name>` to override backend selection for every machine selected
after `--machine` filtering.
ID: systemconfig.cli.surface.cli5

When `--assembler` is absent, the backend MUST be resolved from the selected
machine's `class` field. When `--assembler` is present, the override MUST take
precedence over every selected machine's `class` value. If the named backend is
not registered, each selected machine MUST fail with an assembler diagnostic
before assembly begins whenever the command reaches assembly work. For
`crunch system eval --stop-after=fragments`, assembler lookup is skipped as
specified by CLI-1, so `--assembler` is accepted but ignored and unknown
backend names MUST NOT produce assembler diagnostics in that mode.

#### Scenario: Assembler override wins over machine.class
ID: systemconfig.cli.surface.cli5.scenario

- GIVEN two selected machines whose inventory records set `class = "nixos"`
- AND a registered backend named `container`
- WHEN `crunch system eval <inventory.ncl> --assembler container` runs
- THEN both selected machines use the `container` backend for dry-run assembly
- AND the inventory `class` values are ignored for this invocation

### Requirement: CLI-6 Eval output formats

`crunch system eval` MUST accept exactly two `--format` values: `json` and
`nickel`. The default stdout payload format is `json`.
ID: systemconfig.cli.surface.cli6

Any other `--format` value MUST fail with a CLI diagnostic before evaluation
begins.

The eval-specific `--format` flag controls stdout serialization of the same
`SystemPipelineResult` envelope described by CLI-1. The global `--json` flag
controls stderr diagnostic encoding only. Therefore `crunch system eval --json
--format json` produces JSON on both streams for different purposes, and
`crunch system eval --json --format nickel` is a valid combination once Nickel
stdout output exists.

`--format nickel` MAY be deferred to a follow-on change. If deferred, the CLI
MUST still accept `--format nickel`, MUST fail with a clear not-yet-implemented
error, MUST leave stdout empty, MUST exit non-zero, and MUST continue to use
the ERR-3 stderr diagnostic contract.

#### Scenario: Deferred Nickel output stays explicit
ID: systemconfig.cli.surface.cli6.scenario

- GIVEN `crunch system eval --json <inventory.ncl> --format nickel`
- AND Nickel output has not been implemented yet
- WHEN the command runs
- THEN stdout remains empty
- AND the command exits non-zero with a clear not-yet-implemented diagnostic
- AND stderr still follows the JSON diagnostic contract from ERR-3

#### Scenario: Unsupported eval format is rejected before evaluation
ID: systemconfig.cli.surface.cli6.invalidscenario

- GIVEN `crunch system eval <inventory.ncl> --format yaml`
- WHEN the command parses CLI arguments
- THEN stdout remains empty
- AND the command exits non-zero with a CLI diagnostic before evaluation begins

