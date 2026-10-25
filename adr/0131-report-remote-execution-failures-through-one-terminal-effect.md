# ADR 0131: Report remote execution failures through one terminal effect

## Status

Proposed (2026-10-25). Focused core, application, binary and remote production
tests cover the behavior; acceptance of `separate-remote-build-hexagon` remains
pending.

## Context

The extracted remote effect session marks itself failed after any failed
observation and refuses every later plan with `SessionFailed`. A production
stdio worker records a failed executor launch through that session and then
plans the transport send for its `BuildExecution` error frame. The failed
session refused that send, so the worker exited without reporting and every
failed production remote build reached its client as a transport EOF. The
signed failed-usage report that the frame carries never reached the coordinator.

Repairing the send exposed two more gaps on the same path. The client verified
the failed-usage report against its trusted output keys, and a verification
error replaced the execution failure. The usage-executor child replaces every
failure with a fixed variant, so the worker's `worker_bundle_ref`,
`worker_capture_outcome` and `worker_cleanup_status` fields, which used to ride
in the executor's failure text, no longer reached the coordinator, and
`remote status` could not link a failure to the worker's debug bundle.

The first full ordinary gate run found this through
`oversized_remote_sandbox_artifact_is_rejected_without_rewriting_execution_failure`.
Its positive sibling skipped in development shells because `nix develop` also
sets `NIX_BUILD_TOP`.

## Decision Drivers

- A failed effect is classified through the declared failure transition; the
  shell neither fabricates success nor skips the failed observation.
- Debug and usage evidence never rewrite the original execution result.
- No failure text crosses the executor child boundary.
- Usage accounting records only authenticated reports.

## Decision

A failed `EffectSession` refuses every dependent effect but admits exactly one
more: a `TransportSend` planned through `plan_failure_report`, carrying the
terminal failure report to the peer. A healthy session cannot plan one
(`NoFailureToReport`), and a second is refused (`SessionFailed`).
`crunch_remote_app::send_failure_report` is the application entry point, and the
production worker uses it for the executor-failure error frame.

The executor child keeps its fixed failure variant and adds only an optional
worker failure-debug reference: one `remote-failure-debug:` digest of 64
lowercase hexadecimal characters and two admissible status codes, parsed from
its own failure text. The worker and the client each accept the reference only
if it renders and parses back unchanged, then relay it as
`; worker_bundle_ref=…; worker_capture_outcome=…; worker_cleanup_status=…`
after the fixed codes `remote-executor-failed` and
`remote-worker-execution-failed`. The coordinator parses those fields as before.

The client records a failed-usage report only after authenticating it. When it
cannot, the usage stays unrecorded, the client appends
`failed-usage-unrecorded=<reason>` to the same execution failure, and the
ordinary remote failure path still fails the attempt. The rejection names its
fixed verification code.

Tests assert the execution-failure code and the debug-bundle reference, not the
executor's internal text. The positive capture test skips only inside a Nix
build sandbox: `NIX_BUILD_TOP` set without `IN_NIX_SHELL`.

## Consequences

- A failed production remote build reaches its client as an execution failure,
  `remote status` links it to the worker's failure-debug bundle, and an
  authenticated failed-usage report is recorded.
- Operators read build-failure detail from the worker's debug bundle, not from
  the client message.
- The session state machine gains one flag and one blocker. It is outside the
  Trellis projection, so no verified model changes.
- A client without the builder's key material records no failed usage.
- This record accepts no Cairn change and proves neither remote correctness nor
  that any other failure path reports its outcome.
