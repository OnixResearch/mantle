# Design: Schedule the Tiger Style hardening backlog

## Context

`crunch-tigerstyle check` runs `cargo-tigerstyle check` over the package scope
in `workspace.metadata.tigerstyle.default_scope`. Every lint in `dylint.toml`
is set to `deny`, so any finding fails the flake check.

The lint stops at the first package that fails to compile under lint, so a
single-package run per scope entry is the only way to see every finding. The
2026-10-22 scan over the 72 scoped packages found 477 findings in 24
packages.

## Decisions

### Fix the finding, or record it with a reason

Each finding is handled one of two ways:

1. **Fixed.** The source is changed so the finding no longer applies. This is
   the default and covers the mechanical categories: assertion density,
   compound conditions, numeric unit naming, bool naming, explicit defaults,
   unchecked unsigned arithmetic, function length, nested conditionals,
   parameter count, ambiguous parameters, collection growth and loop bounds.

2. **Recorded.** The finding stays, and the file gains a file-scoped
   `#![allow(tigerstyle::<category>, ...)]` with a `HARDENING-BACKLOG` comment
   naming this change's evidence file. This applies only where removing the
   finding would change observable behaviour, change a published public API,
   or require restructuring outside the scope of the affected package. The
   categories are exactly those present in that file at recording time.

The `HARDENING-BACKLOG` convention is the repository's own, established by the
workspace hardening pass of 2026-09-09 in commit `3f0385b0f`, whose message
records that findings were "fixed or narrowly scoped with recorded reasons".

### The allow block is file-scoped, never global

No allow is added to `dylint.toml`. Every allow lives at the top of the file
that carries the finding, so a later fix in that file removes the allow for
the categories it no longer needs.

### The evidence records what is deferred

`evidence/tigerstyle-backlog-2026-10-22.log` records, for every deferred
finding, the file, the line, the lint category, the reason it was deferred
and the pass it is scheduled for. `evidence/tigerstyle-final-green.txt`
records the gate output that shows the check passing.

## Consequences

- The Tiger Style gate passes, which unblocks the `G2` task of
  `reconcile-main-api-lineages-2026-10-17` and the full flake check with it.
- The deferred findings stay visible in the source, in the evidence and in
  this change, so they are scheduled work rather than hidden debt.
- The consumer-owned lint policy in `dylint.toml` is unchanged, so no new
  category of finding is accepted anywhere.
