# Design: Close data-only dependency exports as rejected at admission

## Context

The original change used the `spec-driven` profile. Its original proposal,
design, tasks, metadata, and spec delta are preserved verbatim in
`evidence/original-scope/proposal.md`, `design.md`, `tasks.md`,
`metadata.json`, and `specs/dependency-exports/spec.md`.

Its delta spec carried four requirements: `exports_record`,
`no_behavior_hooks`, `typed_build_systems`, and `bounded_adoption_gate`.

The only permitted first steps were T1.1, which records the admission
decision, and T1.2, which re-validates the record fields against the named
consumer's real dependency shapes. The 2026-10-07 admission review found no
current consumer. The decision is therefore rejection, and T1.2 has no
consumer to validate against.

Cairn has no withdrawn or rejected change state:

- Archive is blocked while any task is todo, in progress, or unmarked
  (`cairn-core/src/verified/plan.rs:198-210`).
- Task states are only `[x]`, `[~]`, and `[ ]` (`repo/tasks.rs:23-43`).
- The CLI has no reject or withdraw command for changes.

Checking the original tasks would falsely claim work. The change is
therefore rescoped transparently to the decision itself.

## Decisions

### Decision: Preserve the original scope verbatim

Copy the five original artifacts from published main
`e24bbbc2f803f59872e2e59370bd8ce0829c918e` into `evidence/original-scope/`
without edits. Record each file's Git blob identity in
`evidence/original-scope/SOURCE.md`, so `git hash-object` can confirm byte
equality. The original tasks stay unchecked there.

### Decision: Classify the change as `no-spec-delta`

The only thing this change delivers is a lifecycle decision record. No
accepted requirement is added, modified, or removed:

- Removing `specs/dependency-exports/spec.md` keeps the four requirements out
  of `.cairn/specs`.
- Acceptance uses change-local `a[...]` criteria, which sync never promotes.
- `metadata.json` keeps the only admitted profile source value,
  `change_create`. Cairn accepts no other value
  (`artifact_workflow/profile.rs:61-65`).
- The switch from `spec-driven` is disclosed here and in ADR 0109. It is not
  hidden as an original selection.

### Decision: Reference the shared ADR 0109

Both package-layer priors share one admission decision. ADR 0109 records:

- the evidence summary;
- the rejected alternatives;
- the revisit triggers, including acceptance of
  `run-cargo-build-scripts-as-plan-units` with a need for typed exports;
- the non-claims.

The ADR file and its index row land with `add-spec-override-tree`. This
change references the ADR and adds no second copy.

## Failure behavior

- **A consumer appears later.** This archive does not block a new change. The
  new change starts from the preserved original scope and repeats T1.2
  against the consumer's real dependency shapes.
- **An original requirement ID is cited as accepted.** Any
  `mantle.dependency_exports.*` ID cited this way fails Cairn reference
  checks, because the ID never enters `.cairn/specs`.
- **The review receipt is stale.** If the classification review receipt does
  not match the current proposal, design, acceptance, or tasks bytes, the
  tasks gate, sync, and archive all block.

## Risks / Trade-offs

- **The Rust unit-plan lane may need exports later.** ADR 0109 names that
  trigger. The archived original contract stays available, so there is
  little to rebuild.
- **The text search may miss consumers that use other names.** The revisit
  triggers bound this risk.
