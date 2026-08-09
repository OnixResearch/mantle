# ADR 0075: Stream complete root outcomes with canonical summaries

## Status

Accepted

## Context

Mantle can force several selected roots with bounded workers. The pipeline stops new dispatch after the first root error.

This behavior omits independent work and gives machine consumers no complete root accounting. Completion order also varies under parallel execution.

## Decision Drivers

- Give every selected root one terminal outcome.
- Keep root errors separate from shared fatal errors.
- Preserve useful results without reporting full success.
- Keep live records parseable and bounded.
- Keep identity independent from worker completion order.
- Keep the outcome logic pure.
- Keep evaluation, worker, and output effects in shells.
- Preserve existing aggregate output for a bounded period.

## Decision

Mantle will add the `mantle-evaluation-stream-v1` NDJSON contract. Stream mode will write only complete machine records to stdout.

The record kinds are `run-start`, `root-discovered`, `root-terminal`, and `run-summary`. The final writable record must be one `run-summary`.

Mantle will assign each admitted root a source-order sequence and a domain-separated BLAKE3 identity before parallel dispatch. Live terminal records can use completion order. The summary will use source order.

Terminal root states are `succeeded`, `failed`, `worker-lost`, `cancelled`, and `not-started`. Failure scopes are `root-scoped`, `shared-fatal`, `cancellation`, and `coordinator-failure`.

Run dispositions are `success`, `partial`, `failed`, and `cancelled`. Cancellation has precedence. Mixed success and non-cancellation errors produce `partial`.

Evaluation stream process statuses are zero for `success`, two for `partial` or `failed`, and 130 for `cancelled`. Pipeline stream statuses use one for `partial` or `failed`. Output or summary-admission errors use internal status three.

Each record is limited to 128 MiB. Each run is limited to 65,536 roots. Root labels are limited to 1,024 UTF-8 bytes. Diagnostics are limited to 16 KiB after redaction.

Existing aggregate output remains supported in the introduction release and one subsequent minor release. Stream mode requires an explicit option.

## Alternatives Considered

### Stop after the first root error

Rejected. It omits independent outcomes and cannot give complete accounting.

### Emit best-effort records without a final summary

Rejected. A consumer cannot distinguish completion from output loss.

### Use completion order as identity

Rejected. Worker schedules would change canonical identity.

### Import `nix-eval-jobs` behavior

Rejected. Mantle keeps its root model, wire contract, authority, and claim boundaries.

## Consequences

- Independent work can continue after a root-scoped error.
- Shared fatal errors still stop dispatch.
- Partial results remain available with a non-success status.
- Stream readers must reject unknown versions, unknown kinds, duplicate fields, malformed records, and incomplete summaries.
- The stream proves bounded outcome accounting only. It does not prove evaluator, build, cache, or release correctness.
