# Evaluation stream contract

Mantle will expose the `mantle-evaluation-stream-v1` NDJSON contract through an explicit stream option.

This document defines the contract before pipeline dispatch changes.

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

## Compatibility period

Existing aggregate output remains supported in the stream introduction release and one subsequent minor release.

Stream mode requires an explicit option. The aggregate surface does not silently change to NDJSON.

## Reference boundary

The design adapts independent error reporting and JSON-lines framing from `NixOS/nix-eval-jobs` revision `a0cd02231c58974a6b5aaa3712069b071047162e`.

Mantle imports no source or runtime dependency from that GPL-3.0 project. Mantle does not adopt Nix traversal, Hydra aggregates, evaluator callbacks, or Nix private libraries.

## Non-claims

A valid stream proves bounded framing, record shape, terminal accounting, and canonical summary order for supplied facts.

It does not prove evaluator correctness, root independence, build correctness, cache trust, reproducibility, deployment safety, or release eligibility.
