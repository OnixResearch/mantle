# 0123 Retain unwired source-program authors behind item-scoped dead-code allows

## Status

Accepted (2026-10-21).

## Context

The reconciled main tree adds the source-only program authors that the
`source-built-fixed-point` selector will drive: StageX transition, native host
and final phases, Rust successor/minicargo/translation, Mantle metadata and
native leaves, and the project/gateway authority records that bind them. Several
of those helpers are only reachable once the selector is completed (Cairn
`prove-source-built-mantle-fixed-point` I3), which is not yet implemented.

The ordinary first-party gate runs strict Clippy with `-D warnings`, so every
item that no current caller reaches fails the repository's own gate. Deleting
the helpers would discard authored authority derivations that the selector
needs; a crate-level `#![allow(dead_code)]` would also hide unrelated future
dead code.

## Decision Drivers

- The ordinary gate must stay meaningful; it is the only automated guard against
  unreachable code and unused state.
- Authored derivations encode reviewed source authority and must not be
  re-derived ad hoc.
- Every suppression needs an owner, a scope, and a removal trigger.

## Decision

Keep the unwired authoring layers and annotate them with item-scoped
`#[allow(dead_code)]` attributes (or a module-level attribute only when the
whole module is unwired). Each attribute carries the comment naming Cairn
`prove-source-built-mantle-fixed-point` I3 as the consuming change. No
crate-level allow is introduced.

## Consequences

- Strict Clippy continues to flag new unreachable items everywhere else.
- The annotations are explicit debt: I3 must remove each one as it wires the
  corresponding author into the source-only preparation path.
- Removing an annotated item later is a deliberate, reviewable change; nothing
  depends on the suppression for behavior.

Non-claim: the annotations do not assert that the retained authors are correct,
complete, or reachable. They record that the items are authored and pending
wiring.
