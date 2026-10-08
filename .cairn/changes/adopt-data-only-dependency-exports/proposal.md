# Proposal: Close data-only dependency exports as rejected at admission

## Why

`adopt-data-only-dependency-exports` was added on 2026-09-10 as a design
prior. It defined three things for a future Mantle package layer:

- typed per-output exports records;
- a rule that dependencies provide data and never behavior;
- a typed build-system registry.

It blocked every implementation task behind an admission gate (T1.1, T1.2)
that required a named first consumer, target outcome, adoption path, and
maintenance owner. The proposal said: "Immediate consumer: none yet". It
also set the success criterion that the admission record names those facts
"or the change stays unimplemented by rule".

The workspace rule says: "Foundation work must name a current consumer,
target outcome, adoption path, and maintenance owner. Reject infrastructure
without concrete demand." The 2026-10-07 admission review
(`evidence/admission-review-2026-10-07.md`) found no current consumer:

- **`builders/` layer:** used only by examples and tests. No bootstrap recipe
  imports it, and its dependencies only add `/bin` to `PATH`, so there are no
  setup hooks to replace.
- **mantlepkgs catalog:** has no dependency-exports demand.
- **Sibling repositories:** none consumes Mantle dependency exports.
- **`run-cargo-build-scripts-as-plan-units`:** the only self-declared
  candidate. It is unpublished, not started (0/16), and depends on an
  unaccepted parent. Its role-name and pkg-config contract does not need
  tree-derived exports records or a typed build-system registry.

The change is therefore rejected at admission. It was not implemented.

## What Changes

- The original proposal, design, tasks, metadata, and spec delta are
  preserved verbatim under `evidence/original-scope/`, with their source
  commit and Git blob identities.
- This change becomes a decision-only `no-spec-delta` change. Its tasks are:
  - the preservation;
  - the admission review;
  - the ADR 0109 rejection reference;
  - the spec-delta removal;
  - the closure record.
- The spec delta `specs/dependency-exports/spec.md` is removed, so none of
  the `mantle.dependency_exports.*` requirements becomes accepted. Accepted
  specifications do not change, and sync is an explicit no-op.
- ADR 0109 records the rejection, revisit triggers, and non-claims. The ADR
  is shared with `add-spec-override-tree` and lands with that change's
  branch.

## Impact

- **Immediate consumer**: none. That absence is the reason for rejection.
- **Immediate outcome**: an unsupported foundation package is closed with a
  durable, reviewable decision instead of staying open indefinitely.
- **Durable capability**: the reviewed exports and registry contract stays
  recoverable from `evidence/original-scope/` if a revisit trigger in ADR
  0109 occurs. One such trigger is `run-cargo-build-scripts-as-plan-units`
  being accepted and needing typed exports.
- **Maintenance owner**: Mantle maintainers own ADR 0109 and its revisit
  triggers.
- **Compatibility**: no code, Nickel contract, CLI, or accepted specification
  changes. The `builders/` behavior is untouched.

## Non-Goals

- Implementing exports records, the no-behavior rule, the build-system
  registry, or prepare rendering.
- Judging the design quality of the original contract.
- Changing `builders/`, mantlepkgs, the Rust unit-plan changes, or any
  consumer repository.
