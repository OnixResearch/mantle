# Design: surface foreign realization diagnostics

## Context

`ingest_http_nar` is the single HTTP NAR download/ingest step shared by the flat pull
(`pull_single_http_path`) and the strict seeded closure import (`import_planned_http_member`). It
refuses a NAR in six places: NAR URL resolution, redirect status, non-success HTTP status, request
error, unsupported compression, and NAR ingestion failure. Every refusal increments a `PullReport`
skip counter and returns `Ok(None)`; only the flat pull has a use for the counter, and the closure
import turns every `None` into the same cause-free error.

`foreign-import realize` builds through `crunch_pipeline::build_registered_derivations`, whose
`RegisteredBuildResult` already carries `BuildOutcome.log` for executed units and
`FailedGoal.build_log` for a failed root whose output admission failed after execution.
`mantle build` persists those with `build_log::write_log_file` under `<state-dir>/logs`, and
`mantle log` reads that directory through `ctx.resolved_log_dir`.

A live run of the real Octet job on Mantle 0b78fc1b6 exposed the gap the first version of this
decision left: its root build failed twice inside the Lean builder, `mantle log --list` reported
"No build logs found", and `log <drv>` exited 3, while the receipt's `failure.message` held the
whole builder output including the `affinity list` lines. `crunch-build` sets `FailedGoal.build_log`
only when output admission fails after a successful execution; a builder failure leaves its output
in `FailedGoal.error`. Writing only `build_log` therefore dropped precisely the failures an
operator needs to inspect.

## Decisions

### Decision: Refusal is a value, not a missing option

**Choice:** `ingest_http_nar` returns `Result<NarIngestion, Error>` with
`enum NarIngestion { Ingested(IngestedNar), Refused(NarRefusal) }`. `NarRefusal` holds a
`&'static str` class and a `String` detail; `MAX_NAR_REFUSAL_DETAIL_BYTES` bounds the detail, keeping
the longest prefix that ends on a UTF-8 boundary and appending `...` when truncated. Classes are
`nar-url-rejected`, `nar-redirect-rejected`, `nar-http-status`, `nar-request-failed`,
`nar-compression-unsupported`, and `nar-ingestion-failed`.

**Rationale:** the closure import can render `<path>: <class>: <detail>` while the flat pull maps the
same value back onto its existing counters. Two callers, one ingestion path, no duplicated refusal
classification. A class token gives operators and tests a stable anchor; the bounded detail carries
the cause (including the HTTP status) without letting a proxy error page bloat the error.

**Rejected alternatives:** returning `Result<Option<_>, Error>` with an error only for the closure
import (the flat pull would then need a second download path); logging structured fields only
(invisible under `--json`); embedding the detail into the class string (no stable token to match).

### Decision: Realization logs reuse `mantle build`'s writer

**Choice:** a new shell helper creates `<state-dir>/logs` and calls
`build_cmd::write_log` for each `RegisteredBuildResult.outcomes[].log` (`success = true`). For each
`RegisteredBuildResult.failed_roots[]` it writes the retained `build_log` (`success = false`) when
there is one, and otherwise calls the shared `build_cmd::write_failed_goal_log`, the same helper
`mantle build` uses: the same `should_write_failure_log` gate, drv key, `origin …` detail for a
dependency failure, and writer. The derivation basename is the label. It returns
`Vec<DiagnosticPersistenceFailure>`; the outcome carries it to
`run_realize`. In human mode `run_realize` prints the same `WARNING: diagnostic persistence failed`
block as `mantle build`. Under `--json`, where stdout is the receipt itself, it writes each failure as
one JSON object line `{"diagnostic_persistence_failure": {...}}` on stderr, using the struct's
existing serialization. No placeholder log is written for cached or reused outcomes, so a reuse run
leaves an existing file untouched.

**Rationale:** extracting the helper rather than copying it keeps `mantle build`'s output
byte-identical by construction and makes "the log a realization writes for a failure" the same log
`mantle build` writes. The file name, header, timestamp and status semantics stay byte-compatible
with `mantle build`, so `mantle log` needs no change and an operator can quote either. Keeping the
failures out of the receipt preserves receipt v1/v2 identity and keeps `--json` stdout a single
receipt. JSON lines keep `--json` stderr machine-readable, as `AGENTS.md` requires, while making
the failure visible in the mode the pilot runs use.

**Consequence:** a log-write failure never changes the result or the receipt and is always
reported: as a warning in human mode, as a JSON line on stderr under `--json`.

**Rejected alternatives:** adding a receipt field (schema change, and the receipt is published
before this point); failing the realization when a log cannot be written (a full disk would then
discard an otherwise valid realization); writing logs from `run_realize` (the build result does not
cross the effect-port boundary, and `--json` output is emitted there).

## Risks / Trade-offs

- The flat pull's behavior must stay identical: the refusal value is mapped back to the same
  `skipped_*` counters in the same branches, and the existing unit tests for those counters run
  unchanged.
- A detail string copied from a proxy error page could contain newlines; the byte bound keeps the
  error line bounded, and tests pin the `<class>: <detail>` shape rather than exact detail bytes for
  transport errors.

## Evidence

- `crates/crunch-store/src/pull.rs` unit tests: a 503 NAR response reports
  `nar-http-status` with the status in the detail; a NAR whose bytes do not match reports
  `nar-ingestion-failed`; a successful closure import still imports; the flat pull still counts a
  refused NAR as skipped without an error.
- `tests/cache_inputs_build_route.rs`: a successful realize leaves `<state>/logs/<drv>.log` with the
  builder's output and `mantle log <drv>` shows it; a reuse run does not rewrite it.
- `src/foreign_realization_shell.rs` unit tests: a log-directory failure is returned as a diagnostic
  persistence failure instead of failing the realization; a failed root with no retained build log
  writes the `mantle build` failure detail including the `origin …` line; a failure
  `should_write_failure_log` rejects writes nothing; a retained build log still wins.
