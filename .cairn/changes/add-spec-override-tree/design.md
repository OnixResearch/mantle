# Design: Close the spec override tree as rejected at admission

## Context

The original change (preserved verbatim in
`evidence/original-scope/proposal.md`, `design.md`, `tasks.md`,
`metadata.json`, and `specs/spec-override-tree/spec.md`) used the
`spec-driven` profile. It carried a delta spec with four requirements:

- `verb_semantics`
- `path_validation`
- `merge_cost_bound`
- `bounded_adoption_gate`

Its only allowed first step was T1.1, which records the admission decision.
The 2026-10-07 admission review found no consumer, so the decision is
rejection.

Cairn has no withdrawn or rejected change state:

- Archive is blocked while any task is todo, in progress, or unmarked
  (`cairn-core/src/verified/plan.rs:198-210`).
- Task states are only `[x]`, `[~]`, and `[ ]` (`repo/tasks.rs:23-43`).
- The CLI has no reject or withdraw command for changes.

Checking the original implementation tasks would falsely claim work, so the
change is rescoped transparently to the decision itself.

## Decisions

### Decision: Preserve the original scope verbatim

Copy the five original artifacts from published main
`e24bbbc2f803f59872e2e59370bd8ce0829c918e` into `evidence/original-scope/`
without edits. Record each file's Git blob identity in
`evidence/original-scope/SOURCE.md`; a reader can confirm byte equality with
`git hash-object`. The original tasks there stay unchecked, because none was
performed.

### Decision: Classify the change as `no-spec-delta`

The only behavior this change ships is a lifecycle decision record. No
accepted requirement is added, modified, or removed. Removing
`specs/spec-override-tree/spec.md` keeps the four
`mantle.spec_override_tree.*` requirements out of `.cairn/specs`.
Acceptance uses change-local `a[...]` criteria, which sync never promotes.
`metadata.json` keeps `change_create`, the only profile source value Cairn
admits (`artifact_workflow/profile.rs:61-65`). The switch from `spec-driven`
is disclosed here and in ADR 0109, not presented as an original selection.

### Decision: Record the rejection once in ADR 0109

Both package-layer priors share one admission decision, so one ADR records:

- the evidence summary;
- the rejected alternatives (implement now, park as `deferred`, delete);
- the revisit triggers;
- the non-claims.

This change adds ADR 0109 and its index row.

## Failure behavior

- If any later reviewer finds a current consumer, this archive does not
  block them. They open a new change from the preserved original scope and
  re-review the contract against that consumer.
- If any original `mantle.spec_override_tree.*` ID is cited as accepted,
  it fails Cairn reference checks, because the ID never enters
  `.cairn/specs`.
- If the classification review receipt does not match the current proposal,
  design, acceptance, or tasks bytes, the tasks gate, sync, and archive all
  block.

## Risks / Trade-offs

- A rejected design prior could be rediscovered and re-proposed without
  context. ADR 0109 and the archived change keep the original text and
  evidence next to the decision.
- The text search may have missed a consumer that uses unrelated names. The
  revisit triggers bound this risk.
