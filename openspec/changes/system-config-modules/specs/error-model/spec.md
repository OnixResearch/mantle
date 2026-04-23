## ADDED Requirements

### Requirement: ERR-1 Error and warning taxonomy

The system-config pipeline MUST expose a shared error taxonomy rooted in a
`SystemConfigError` enum plus a shared warning taxonomy rooted in a
`SystemConfigWarning` enum.
ID: systemconfig.error.model.err1

`SystemConfigError` MUST provide layer-specific variants that carry enough
context for a human-readable diagnostic without re-reading source files:
`Cli`, `Loader`, `Inventory`, `CrossRef`, `Eval`, `Fragment`, and
`Assembler`. `SystemConfigWarning` MUST provide at least warning kinds needed by
the current pipeline semantics, including orphan-provider consumption. Both
taxonomies MUST preserve provenance fields such as `module_name`,
`machine_name`, and `field_path` when those fields are applicable.

#### Scenario: Layered diagnostics preserve provenance
ID: systemconfig.error.model.err1.scenario

- GIVEN a module contract failure during evaluation
- WHEN the pipeline records the failure
- THEN the result includes a `SystemConfigError::Eval` value
- AND the diagnostic names the module and the failing field path when known

### Requirement: ERR-2 Propagation policy

The pipeline MUST apply a consistent propagation policy across all layers.
ID: systemconfig.error.model.err2

Loader failures are fail-open per module: a module that fails structural
validation is excluded from later stages, and dependent modules fail with
chained diagnostics that reference the originating loader failure. If an
inventory service names a module that was excluded by loader validation, later
cross-reference handling MUST report that condition as a chained dependency-on-
failed-module diagnostic rather than as a bare missing-module error. Inventory
failures are fail-closed: when the inventory is structurally invalid, the
pipeline stops before module evaluation. Evaluator, fragment, and assembler
failures are fail-open per independent machine or independent dependency
subgraph.

#### Scenario: Inventory validation stops the whole pipeline
ID: systemconfig.error.model.err2.scenario

- GIVEN an inventory that is structurally invalid
- WHEN a system command starts
- THEN the command returns an inventory error before any module evaluation
- AND no machine outcomes are produced

### Requirement: ERR-3 Structured stderr diagnostics

When crunch's global `--json` flag is active, warnings and errors MUST be
emitted to stderr as one structured JSON diagnostic object per line.
ID: systemconfig.error.model.err3

Each JSON diagnostic object MUST contain `severity`, `layer`, `message`, and
`detail` fields, plus `module` and `machine` fields when applicable. CLI-
originated failures such as invalid flags or deferred feature errors MUST use
`layer = "cli"`. When the global `--json` flag is not active, stderr MUST
remain human-readable text.
Stdout payload shape is defined by the CLI surface requirements and MUST remain
separate from stderr diagnostics in both modes. Diagnostic emission order on
stderr MUST be deterministic: sort first by machine name with non-machine
(fatal pre-machine) diagnostics before machine-scoped diagnostics, then by
module name when present, then by severity with `error` before `warning`, and
finally by stable message text as a last tiebreak.

#### Scenario: JSON diagnostics stay on stderr
ID: systemconfig.error.model.err3.scenario

- GIVEN `crunch system build --json <inventory.ncl>`
- AND one selected machine fails during assembly
- WHEN the command exits
- THEN stderr contains a JSON diagnostic object with `severity = "error"`
- AND stdout remains reserved for the command's primary result payload

### Requirement: ERR-4 Top-level result shape

Non-fatal pipeline execution MUST produce a `SystemPipelineResult` value that
carries successful machine outcomes together with collected diagnostics before
any CLI-specific rendering is applied.
ID: systemconfig.error.model.err4

The result type MUST contain a `machines` map plus separate `errors` and
`warnings` collections:

```text
struct SystemPipelineResult {
    machines: BTreeMap<String, MachineOutcome>,
    errors: Vec<SystemConfigError>,
    warnings: Vec<SystemConfigWarning>,
}
```

`MachineOutcome` MUST define the stable terminal variants consumed by CLI-1 and
CLI-2: `Fragments { merged_config }` for `--stop-after=fragments`,
`Derivations { derivations }` for eval dry-run derivations,
`Build { reports }` for one or more machine-level `crunch-build-report-v1`
payloads produced from that machine's assembled derivations, and `Failed` for
selected machines that do not reach a successful payload.

Every selected machine MUST appear in the `machines` map with one terminal
`MachineOutcome` variant so CLI summaries can distinguish success from failure
without inferring omission from missing keys. A selected machine MUST use
`MachineOutcome::Failed` when any module instance or later stage error prevents
that machine from producing a complete payload for the requested phase; warning-
only conditions do not force failure by themselves.

When serialized to JSON for CLI stdout, each machine outcome MUST be an object
with a `status` discriminator field whose value is one of `fragments`,
`derivations`, `build`, or `failed`. The payload fields are fixed by status:
`fragments` carries `merged_config`, `derivations` carries `derivations`,
`build` carries `reports`, and `failed` carries no success payload.

When serialized inside stdout `SystemPipelineResult` envelopes, `errors` and
`warnings` entries MUST use the same object shape as ERR-3 diagnostics:
`severity`, `layer`, `message`, and `detail`, plus `module` and `machine`
fields when applicable. The `errors` and `warnings` vectors MUST each use the
same deterministic ordering rule as ERR-3 stderr emission so repeated runs over
the same inputs produce identical stdout collection order.

Fatal inventory-level validation failures MAY return a top-level
`Err(Vec<SystemConfigError>)` instead of a `SystemPipelineResult`, because no
machine-level work can proceed in that case.

#### Scenario: Partial success returns machines plus diagnostics
ID: systemconfig.error.model.err4.scenario

- GIVEN two machines where one succeeds and one fails after module evaluation
- WHEN the system command completes
- THEN `SystemPipelineResult.machines` contains the successful machine outcome
- AND `SystemPipelineResult.machines` contains `MachineOutcome::Failed` for the
  failed selected machine
- AND `SystemPipelineResult.errors` contains the failure diagnostic
- AND `SystemPipelineResult.warnings` remains available for non-fatal warnings
