# Proposal: Close the spec override tree as rejected at admission

## Why

`add-spec-override-tree` was added on 2026-09-10 as a design prior. It
defined checked override verbs (`set`, `append`, `prepend`, `merge`,
`remove`, `edit`) over package specs for a future Mantle package layer. It
blocked every implementation task behind an admission gate that required a
named first consumer, outcome, adoption path, and maintenance owner. The
proposal itself said: "Immediate consumer: none yet — design prior".

The workspace rule says: "Foundation work must name a current consumer,
target outcome, adoption path, and maintenance owner. Reject infrastructure
without concrete demand." The 2026-10-07 admission review
(`evidence/admission-review-2026-10-07.md`) found none of the four facts:

- No Mantle package layer exists.
- `overrideAttrs` has only example callers.
- mantlepkgs uses `Variant` records.
- No sibling repository consumes Mantle package specs.

The change is therefore rejected at admission. It was not implemented.

## What Changes

- The original proposal, design, tasks, metadata, and spec delta are
  preserved verbatim under `evidence/original-scope/`, with their source
  commit and Git blob identities.
- This change becomes a decision-only `no-spec-delta` change. Its tasks are
  the preservation, the admission review, the ADR 0109 rejection record, the
  spec-delta removal, and the closure record.
- The spec delta `specs/spec-override-tree/spec.md` is removed, so none of
  the `mantle.spec_override_tree.*` requirements becomes accepted. Accepted
  specifications do not change: sync is an explicit no-op.
- ADR 0109, shared with `adopt-data-only-dependency-exports`, records the
  rejection, the revisit triggers, and the non-claims.

## Impact

- **Immediate consumer**: none. That absence is the reason for rejection.
- **Immediate outcome**: an unsupported foundation package is closed with a
  durable, reviewable decision instead of staying open indefinitely.
- **Durable capability**: the reviewed override contract stays recoverable
  from `evidence/original-scope/` if a revisit trigger in ADR 0109 occurs.
- **Maintenance owner**: Mantle maintainers own ADR 0109 and its revisit
  triggers.
- **Compatibility**: no code, Nickel contract, CLI, or accepted specification
  changes. Existing `overrideAttrs` behavior is untouched.

## Non-Goals

- Implementing any override verb, path validation, layer merge, or cost
  measurement.
- Judging the design quality of the original contract.
- Changing `builders/`, mantlepkgs, or any consumer repository.
