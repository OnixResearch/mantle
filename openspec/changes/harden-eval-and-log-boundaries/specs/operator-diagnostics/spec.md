## ADDED Requirements

### Requirement: Diagnostic persistence failures are surfaced explicitly

Operator-facing diagnostic persistence failures MUST be surfaced explicitly
instead of being silently ignored.

When crunch cannot persist a build log or another operator-facing diagnostic
artifact, it MUST either return a typed error or emit an explicit warning or
structured diagnostic event that names the failed operation.

Crunch MUST NOT report a saved-log path for an artifact that was not actually
written.

#### Scenario: Build log write failure is reported without a fake saved path

- GIVEN a build or failure path where crunch attempts to persist a build log
- AND the target log directory is unwritable or the write fails
- WHEN crunch reports the result to the operator
- THEN the operator-visible output reports the log-write failure explicitly
- AND no fake saved-log path is emitted

#### Scenario: Successful persistence still reports the real saved path

- GIVEN a build or failure path where the log write succeeds
- WHEN crunch reports the result to the operator
- THEN it may emit the saved-log path
- AND that path names an artifact that actually exists on disk
