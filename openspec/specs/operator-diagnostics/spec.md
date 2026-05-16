# Operator Diagnostics Specification

## Purpose

Defines operator-facing diagnostic surfaces for preflight checks, build-plan
preview, failure reporting, and persisted diagnostic artifacts.
## Requirements
### Requirement: Doctor preflight command

The CLI MUST provide a `mantle doctor` command for no-mutate preflight checks.

`mantle doctor` MUST report whether the current host can satisfy the selected
mantle workflow prerequisites without starting a build or mutating store state.

#### Scenario: Doctor reports missing prerequisite clearly

- GIVEN the host is missing a required build prerequisite such as `bwrap`
- WHEN `mantle doctor` runs for a build-capable workflow
- THEN the command exits non-zero
- AND it names the missing prerequisite
- AND it does not start a build or mutate store state

#### Scenario: Doctor reports usable environment

- GIVEN the host satisfies the selected workflow prerequisites
- WHEN `mantle doctor` runs
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
- THEN mantle reports a planned action for each root
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
- WHEN mantle emits the JSON failure report
- THEN the report includes the failing root
- AND it includes the failing phase and error class
- AND it includes the saved log path

#### Scenario: Human failure summary matches JSON facts

- GIVEN the same build failure
- WHEN mantle prints the human-readable failure summary
- THEN it names the same failing root and phase as the JSON report
- AND it points the operator at the same saved log path when one exists

#### Scenario: Pre-build failure omits log path cleanly

- GIVEN a failure that occurs before any saved build log is written
- WHEN mantle emits the human or JSON failure report
- THEN the failing root, phase, and error class are still reported
- AND the saved log path field is omitted

### Requirement: Diagnostic persistence failures are surfaced explicitly

Operator-facing diagnostic persistence failures MUST be surfaced explicitly
instead of being silently ignored.

When mantle cannot persist a build log or another operator-facing diagnostic
artifact, it MUST either return a typed error or emit an explicit warning or
structured diagnostic event that names the failed operation.

Mantle MUST NOT report a saved-log path for an artifact that was not actually
written.

#### Scenario: Build log write failure is reported without a fake saved path

- GIVEN a build or failure path where mantle attempts to persist a build log
- AND the target log directory is unwritable or the write fails
- WHEN mantle reports the result to the operator
- THEN the operator-visible output reports the log-write failure explicitly
- AND no fake saved-log path is emitted

#### Scenario: Successful persistence still reports the real saved path

- GIVEN a build or failure path where the log write succeeds
- WHEN mantle reports the result to the operator
- THEN it may emit the saved-log path
- AND that path names an artifact that actually exists on disk

### Requirement: Mantle transcripts MUST provide executable markdown repros [r[operator-diagnostics.mantle-transcripts]]

Mantle MUST define an executable markdown transcript format for CLI workflows. The format MUST support Mantle command blocks (`mantle`), expected output checks (`expect` and `expect:json`), expected-error blocks (`mantle:error`), hidden setup blocks (`setup:hide`), hidden cleanup blocks (`cleanup:hide`), and transcript options (`transcript:options`). Running a transcript MUST produce a durable output artifact summarizing command results.

#### Scenario: Expected error lets transcript continue [r[operator-diagnostics.mantle-transcripts.expected-error]]

- GIVEN a transcript stanza marked as an expected error
- WHEN the Mantle command in that stanza fails with the expected diagnostic fragment
- THEN the transcript runner records the stanza as passed
- AND subsequent stanzas continue executing

#### Scenario: Transcript defaults to isolated state [r[operator-diagnostics.mantle-transcripts.isolated]]

- GIVEN a transcript without an in-place marker
- WHEN the runner executes it
- THEN the runner uses a fresh temporary store and state directory
- AND the operator's default store/state are not mutated

#### Scenario: In-place transcripts require explicit caller approval [r[operator-diagnostics.mantle-transcripts.in-place-opt-in]]

- GIVEN a transcript declares `in_place: true` in `transcript:options`
- WHEN the runner is invoked without an explicit in-place allow flag
- THEN the runner rejects the transcript before running command blocks
- AND it reports that in-place state mutation needs operator approval

#### Scenario: Hidden setup is not an operator step [r[operator-diagnostics.mantle-transcripts.hidden-setup]]

- GIVEN a transcript contains `setup:hide` before a visible command
- WHEN the runner executes the transcript
- THEN hidden setup output is captured in the evidence artifact
- AND the hidden setup command is not rendered as a user-facing operator step

#### Scenario: Output matching uses normalized fragments [r[operator-diagnostics.mantle-transcripts.normalized-output]]

- GIVEN command output contains temporary store/state paths or platform-specific line endings
- WHEN an `expect` block checks stable output fragments
- THEN the runner matches against normalized output
- AND raw stdout and stderr remain available in the evidence artifact

### Requirement: Operator diagnostics MUST expose semantic graph queries [r[operator-diagnostics.semantic-graph-queries]]

Mantle MUST expose operator-facing query commands for the semantic build graph. At minimum, Mantle MUST provide a graph view for a selected root, a why/explain view for one output or proof claim, and a dependents view for a selected identity. Human-readable and JSON output MUST expose the same core facts.

#### Scenario: Why query explains an output [r[operator-diagnostics.semantic-graph-queries.why]]

- GIVEN an output produced by a graph-capable build
- WHEN an operator runs `mantle why <output-or-identity>`
- THEN Mantle reports the producing recipe identity
- AND it reports relevant source, provider, sandbox, and proof receipt identities when present

#### Scenario: Incomplete graph is reported explicitly [r[operator-diagnostics.semantic-graph-queries.incomplete]]

- GIVEN an output that lacks required graph records
- WHEN an operator runs a graph query for that output
- THEN Mantle reports an incomplete-graph diagnostic
- AND it does not invent missing source, recipe, proof, or witness edges

