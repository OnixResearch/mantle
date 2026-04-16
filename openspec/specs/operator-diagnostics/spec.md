## ADDED Requirements

### Requirement: Doctor preflight command

The CLI MUST provide a `crunch doctor` command for no-mutate preflight checks.

`crunch doctor` MUST report whether the current host can satisfy the selected
crunch workflow prerequisites without starting a build or mutating store state.

#### Scenario: Doctor reports missing prerequisite clearly

- GIVEN the host is missing a required build prerequisite such as `bwrap`
- WHEN `crunch doctor` runs for a build-capable workflow
- THEN the command exits non-zero
- AND it names the missing prerequisite
- AND it does not start a build or mutate store state

#### Scenario: Doctor reports usable environment

- GIVEN the host satisfies the selected workflow prerequisites
- WHEN `crunch doctor` runs
- THEN the command reports success
- AND the report identifies the checked workflow profile

### Requirement: Build plan preview is side-effect free

The CLI MUST provide a build-plan preview mode that reports per-root planned
execution without dispatching builds or mutating local store state.

At minimum the plan output MUST distinguish whether a root is expected to be
reused from local cache, substituted from a cache, built locally, or blocked by
preflight failure.

#### Scenario: Plan mode reports intended action for each root

- GIVEN a build input with multiple roots in different states
- WHEN the operator runs the build-plan preview
- THEN crunch reports a planned action for each root
- AND it does not start sandboxed builds or substitution downloads

### Requirement: Build failures emit typed diagnostics

Build-entry failures MUST emit a typed diagnostic envelope that records the
failing root, the failing phase, the error class, and the saved log path when
one exists.

The saved log path field MUST be optional and omitted when no saved log file was
written for that failure.

Human-readable output and JSON output MUST expose the same facts.

#### Scenario: JSON failure identifies root and log path

- GIVEN a build that fails after writing a saved log
- WHEN crunch emits the JSON failure report
- THEN the report includes the failing root
- AND it includes the failing phase and error class
- AND it includes the saved log path

#### Scenario: Human failure summary matches JSON facts

- GIVEN the same build failure
- WHEN crunch prints the human-readable failure summary
- THEN it names the same failing root and phase as the JSON report
- AND it points the operator at the same saved log path when one exists

#### Scenario: Pre-build failure omits log path cleanly

- GIVEN a failure that occurs before any saved build log is written
- WHEN crunch emits the human or JSON failure report
- THEN the failing root, phase, and error class are still reported
- AND the saved log path field is omitted
