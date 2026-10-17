# ADR 0109: Reject package-layer design priors that have no consumer

## Status

Accepted on 2026-10-08. This ADR closes `add-spec-override-tree` and
`adopt-data-only-dependency-exports` as rejected at admission. Neither change
was implemented. The rejection was decided by the 2026-10-07 Mantle Cairn
finish campaign, which chose to close both changes as decision-only no-spec
changes.

## Context

Two change packages were added on 2026-09-10 as design priors from the
reviewed `mic92/repkgs` reference (commit `1cd7b8b`):

- `add-spec-override-tree` proposes checked override verbs over package specs
  for a future Mantle package layer.
- `adopt-data-only-dependency-exports` proposes typed per-output exports
  records and a typed build-system registry for that same future layer.

Both proposals said: "Immediate consumer: none yet". Both blocked every
implementation task behind an admission gate. That gate required a recorded
decision naming the first consumer, target outcome, adoption path, and
maintenance owner:

- `mantle.spec_override_tree.bounded_adoption_gate`
- `mantle.dependency_exports.bounded_adoption_gate`

The workspace rule (`../../AGENTS.md`, "Shared conventions") says:
"Foundation work must name a current consumer, target outcome, adoption path,
and maintenance owner. Reject infrastructure without concrete demand."

The admission review on 2026-10-07 and 2026-10-08 found no current consumer
in Mantle or in any sibling repository. The full search transcript is in each
change's `evidence/admission-review-2026-10-07.md`. In summary:

- Mantle has no package-layer family, active or archived.
- `builders/` is used only by examples and tests: `examples/package-set.ncl`
  and `tests/integration.rs`.
- None of the 185 `bootstrap/*.ncl` recipes imports `builders/`.
- `overrideAttrs` (`builders/mk_derivation.ncl:262`) is called only by
  `examples/override.ncl:28` and described in `examples/catalog.ncl:276`.
- Builder dependencies only add `/bin` to `PATH`
  (`builders/mk_derivation.ncl:39-45`). There are no setup hooks for an
  exports record to replace.
- mantlepkgs handles variation with identity-bound `Variant` records
  (`mantlepkgs/domains/contracts.ncl:60,107,115`).
- No Nickel file in onixos, onixpkgs, kiln, aspen, kamacite, lattice, site,
  trellis, tile, chaoscontrol, cairn, octet, nickel-export, animus, basalt,
  valence, or artifact imports Mantle builders, uses `overrideAttrs`, or
  names either contract. The only text match was
  `onixos/lib/exports.ncl:79-82`, an OnixOS service-export helper, which is
  unrelated.
- The only self-declared candidate is the unpublished
  `run-cargo-build-scripts-as-plan-units` change. It exists only in a
  maintainer's local checkout, has 0/16 tasks done, and depends on an
  unaccepted parent. Its contract uses role-named native inputs and
  pkg-config directories, and it already forbids dependency hooks.

## Decision drivers

- No infrastructure without concrete demand.
- No accepted requirement may describe behavior nobody implements or consumes.
- The reviewed design work stays recoverable without being treated as
  accepted.
- Task checkmarks must describe only work that was actually done.

## Decision

Close both changes as rejected at admission, through Cairn's supported
`no-spec-delta` profile:

1. The original proposal, design, tasks, metadata, and spec delta of each
   change are preserved verbatim under `evidence/original-scope/`. Their Git
   blob identities equal those at published main `e24bbbc2f803`.
2. The spec deltas are removed. No `mantle.spec_override_tree.*` or
   `mantle.dependency_exports.*` requirement becomes accepted, and sync is
   an explicit no-op.
3. Each change is rewritten as a decision-only change. Its tasks cover the
   preservation, the admission review, this ADR, the no-spec conversion, and
   the closure record. It is then archived. The workflow profile switches
   from `spec-driven` to `no-spec-delta`. The metadata keeps the source value
   `change_create`, the only one Cairn admits
   (`cairn-core/src/verified/artifact_workflow/profile.rs:61-65`). The switch
   is disclosed in each change's design and here; it is not presented as the
   original selection.

Cairn has no withdrawn or rejected change state. Archive requires every task
to be done (`cairn-core/src/verified/plan.rs:198-210`), and task states are
only `[x]`, `[~]`, and `[ ]` (`repo/tasks.rs:23-43`). The decision-only
rescope is therefore the supported way to close these changes without fake
checkmarks.

## Rejected alternatives

- **Implement now.** That would build infrastructure without demand,
  contrary to the workspace rule.
- **Keep the changes active with `"status": "deferred"`.** This is allowed by
  policy, but it leaves permanently open packages that count as active work.
- **Delete the change directories.** That leaves no Cairn lifecycle record
  of the decision.

## Revisit triggers

Open a new change from the preserved `evidence/original-scope/` text, after
re-reviewing it against the consumer, when any of these occurs:

- A named consumer with a maintenance owner needs package-spec overrides.
  Examples: a Mantle package layer is admitted, or `overrideAttrs`, a
  successor, or the mantlepkgs `Variant` mechanism gains a non-example caller
  that needs layered verb edits.
- `run-cargo-build-scripts-as-plan-units` is accepted, and its T1.2 or T4.2
  needs export data derived from a dependency's output tree, or typed
  build-system options, that its role-name and pkg-config declarations cannot
  express.
- A builder family adopted by bootstrap recipes or mantlepkgs starts running
  dependency-provided setup behavior that a data-only exports record would
  replace.
- A sibling repository (onixos, onixpkgs, kiln, or another) consumes Mantle
  package specs and asks for checked overrides or data-only dependency
  interfaces.

## Consequences and non-claims

- The `repkgs` review evidence and both original contracts remain readable in
  the archived changes. They are design inputs, not accepted Mantle behavior.
- This ADR does not claim that the override-tree or exports designs are wrong
  or inferior. It records only that there is no current demand.
- It does not claim that existing `overrideAttrs` or builder behavior is
  correct, complete, or cheap. Neither is changed.
- It does not prove that no consumer exists anywhere outside the searched
  repositories and revisions listed in the admission evidence.
- Archiving the two decision-only changes proves only Cairn lifecycle
  conformance of the closure record. It does not prove implementation,
  evaluation cost, or path validation.
