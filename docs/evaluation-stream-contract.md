# Evaluation stream contract

Mantle will expose the `mantle-evaluation-stream-v1` NDJSON contract through an explicit stream option.

Use `mantle build --evaluation-stream <source.ncl>` to select the stream explicitly.

`--evaluation-stream` cannot be combined with `--json`, `--fix`, `--plan`, or remote build dispatch. Existing `--json build` output remains available during the compatibility period.

## Optional live coordination view

The NDJSON contract above is a **recorded stream**, not a subscription. For a
separate, transient current-state view, start `mantle remote live serve --socket
/run/user/<uid>/mantle-live.sock` and subscribe with `mantle remote live
subscribe --socket /run/user/<uid>/mantle-live.sock [--owner RUN_ID] [--job
ROOT_ID_OR_DRV_PATH_OR_JOB_ID] [--kind goal]`. Repeat each filter flag
to select several values; without a filter, all current facts match. The
subscription emits newline-delimited `mantle-live-build-protocol-v1`
`snapshot-start`, matching
`publish` facts, `snapshot-end`, then later `publish`/`retract` changes. Job
filters exclude worker-presence facts, which have no job id.

Set `MANTLE_LIVE_STATE_SOCKET` to that socket path when invoking `mantle build`
to opt into best-effort publication. A missing, failed, or slow endpoint
degrades observation only; it cannot reject a build or affect output admission.
The daemon serves at most 4,096 current facts and 32 subscribers; each
subscriber has a 64-event pending queue, and an oversized filter or frame is
rejected. Facts are owner-scoped, retracted when the owner disconnects, and
not persisted across a daemon restart. A subscriber MUST discard its prior
facts on disconnect and request a fresh snapshot after reconnecting.

The live `owner_run_id` names one build invocation. It is distinct from the
content-addressed evaluation-stream `run_id`, which stays unchanged even for
concurrent identical inputs; use the live owner id to filter one invocation.

This view does not change `--evaluation-stream`, `--json build`, stdout framing,
or the meaning of terminal root states. A live terminal fact is not a build
receipt or evidence of output trust. The opt-in publisher observes selected
evaluation-root discovery/terminal events and, separately, actual in-process
Worker scheduler goals (identified by their `.drv` store path) when admitted,
dispatched or completed. It also projects accepted remote coordinator states.
The evaluation stream alone is never used to infer dispatch or worker presence;
local Worker dispatch carries no remote attempt, fence or resource lease.

## Record framing

Stdout contains one complete JSON object per line. Logs and human diagnostics use stderr.

A record cannot exceed 134,217,728 bytes, including its newline. The stream rejects blank lines, malformed JSON, duplicate fields, and trailing data.

The supported record kinds are:

1. `run-start`
2. `root-discovered`
3. `root-terminal`
4. `run-summary`

`run-start` is first. `run-summary` is last. A complete writable stream contains exactly one record of each kind required by its selected roots.

## Bounds

| Bound | Value | Policy |
|---|---:|---|
| `MAX_STREAM_RECORD_BYTES` | 134,217,728 | Reject the record before publication. |
| `MAX_SELECTED_ROOTS` | 65,536 | Reject root-set admission. |
| `MAX_ROOT_LABEL_BYTES` | 1,024 | Reject the root before dispatch. |
| `MAX_DIAGNOSTIC_BYTES` | 16,384 | Redact, then truncate with `diagnostic_truncated = true`. |
| `MAX_SAFE_REFERENCE_BYTES` | 4,096 | Reject the record value. |

All byte limits use UTF-8 bytes. Sequence values start at zero and remain less than `MAX_SELECTED_ROOTS`.

Diagnostic admission redacts absolute paths, credential-shaped tokens, and standalone hexadecimal digests before truncation. It normalizes diagnostic whitespace.

## Root identity and order

Mantle assigns the root sequence before dispatch. The sequence follows the admitted source order.

The root-identity domain is `mantle-evaluation-stream-root-v1\0`.

The preimage contains these fields in this order:

- stream schema
- evaluator cohort
- source BLAKE3 identity
- selector
- root label
- source-order sequence

Each text field uses a four-byte, big-endian length followed by its UTF-8 bytes. The sequence uses one four-byte, big-endian integer.

The run-identity domain is `mantle-evaluation-stream-run-v1\0`. Its preimage contains the schema, cohort, source identity, selector, and ordered root identities.

Live `root-terminal` records can use completion order. The `run-summary.roots` array uses ascending sequence order.

## Terminal states and failure scope

Terminal root states are:

- `succeeded`
- `failed`
- `worker-lost`
- `cancelled`
- `not-started`

Failure scopes are:

- `root-scoped`
- `shared-fatal`
- `cancellation`
- `coordinator-failure`

A successful root has no failure scope or diagnostic. A cancelled root uses `cancellation`.

Terminal phases are `evaluation`, `conversion`, `build`, and `coordination`. The pipeline currently reports successful derivation conversion with `terminal_phase = "conversion"`.

A `worker-lost` root uses `coordinator-failure`. A `not-started` root uses the scope that stopped dispatch.

Each root can have one `root-terminal` record. A repeated terminal record is a contract error.

## Run disposition

The summary derives its disposition from complete terminal facts:

- `success`: all roots succeeded.
- `partial`: at least one root succeeded and another root failed without cancellation.
- `failed`: no root succeeded and no root was cancelled.
- `cancelled`: at least one root was cancelled or cancellation stopped dispatch.

The selected-root count and terminal-root count must match. A missing outcome prevents summary admission.

## Process status

| Mode | `success` | `partial` | `failed` | `cancelled` |
|---|---:|---:|---:|---:|
| Evaluation stream | 0 | 2 | 2 | 130 |
| Pipeline stream | 0 | 1 | 1 | 130 |

A broken stream, rejected summary, or internal coordinator error returns status 3. These errors do not produce a successful completion claim.

A build can fail after its selected roots convert successfully. In that case, the evaluation summary can report `success`, but the pipeline process returns status 1. The stream does not claim build correctness.

## Output failure policy

The stream shell serializes and checks each record before writing it. It flushes after each complete NDJSON line.

If a write or flush fails, the shell requests evaluation cancellation and returns status 3. It does not drain unbounded work or emit a replacement summary.

If the pipeline does not supply an admitted `run-summary`, the shell returns status 3. Earlier records do not establish successful completion.

## Operator interruption

Signal listeners are active only during explicit evaluation-stream mode. On Unix, the first `SIGINT` or `SIGTERM` requests cooperative cancellation. Other targets use the portable Ctrl-C listener.

After the first signal, the shell continues bounded record draining. A writable stream ends with a cancelled `run-summary` and process status 130.

A repeated interruption forces process status 130 without claiming a complete summary. This path lets an operator stop a blocked graceful shutdown.

## Compatibility period

Existing aggregate output remains supported in the stream introduction release and one subsequent minor release.

Stream mode requires an explicit option. The aggregate surface does not silently change to NDJSON.

## Reference boundary

The design adapts independent error reporting and JSON-lines framing from `NixOS/nix-eval-jobs` revision `a0cd02231c58974a6b5aaa3712069b071047162e`.

Mantle imports no source or runtime dependency from that GPL-3.0 project. Mantle does not adopt Nix traversal, Hydra aggregates, evaluator callbacks, or Nix private libraries.

## Non-claims

A valid stream proves bounded framing, record shape, terminal accounting, and canonical summary order for supplied facts.

It does not prove evaluator correctness, root independence, build correctness, cache trust, reproducibility, deployment safety, or release eligibility.
