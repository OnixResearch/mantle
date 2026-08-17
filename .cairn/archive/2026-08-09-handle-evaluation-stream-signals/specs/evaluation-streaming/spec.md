# Evaluation Streaming Specification Delta

## ADDED Requirements

### Requirement: Process signals drive bounded stream cancellation

r[evaluation_streaming.signal_cancellation] During explicit evaluation-stream mode, Mantle MUST translate the first supported process interruption signal into cooperative cancellation. It MUST keep bounded output active until it emits an admitted cancelled summary or encounters a separate terminal output failure. A repeated interruption MAY force immediate termination, but it MUST NOT report success.

#### Scenario: Operator sends SIGINT

- **GIVEN** evaluation-stream mode has emitted `run-start` and selected roots remain unresolved
- **WHEN** the process receives `SIGINT`
- **THEN** Mantle MUST request cooperative evaluation cancellation
- **AND** the writable stream MUST end with one cancelled `run-summary` and process status 130

#### Scenario: Operator sends SIGTERM

- **GIVEN** evaluation-stream mode has emitted `run-start` and selected roots remain unresolved
- **WHEN** the process receives `SIGTERM` on a supported Unix target
- **THEN** Mantle MUST apply the same cooperative cancellation policy as `SIGINT`
- **AND** the writable stream MUST end with one cancelled `run-summary` and process status 130

#### Scenario: Operator repeats interruption

- **GIVEN** the first interruption already requested cooperative cancellation
- **WHEN** another supported interruption arrives before completion
- **THEN** Mantle MAY stop waiting for the pipeline and return the cancelled process status
- **AND** it MUST NOT emit or report a successful summary

#### Scenario: Stream mode is not selected

- **GIVEN** a Mantle command does not select evaluation-stream mode
- **WHEN** command execution starts
- **THEN** this stream-specific signal monitor MUST NOT be installed
- **AND** existing command signal behavior MUST remain unchanged
