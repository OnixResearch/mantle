# Change: Schedule the Tiger Style hardening backlog

## Why

The `nix flake check` gate runs `crunch-tigerstyle check` over the scope
declared in `Cargo.toml` under `workspace.metadata.tigerstyle.default_scope`.
That scope covers 72 packages. The gate currently reports findings in 24 of
them, because the workspace Tiger Style hardening pass of 2026-09-09 covered
`crunch-store`, `crunch-build`, `crunch-pipeline`, `crunch-release-core` and
the `mantle` binary, and the adapter and contract crates added since that
pass were never brought under the same discipline.

The reconciliation change `reconcile-main-api-lineages-2026-10-17` records the
full current-system flake check, including Tiger Style, as unexecuted. Its
`G2` task cannot be closed while the gate fails, and the flake check cannot
pass while the gate fails. This change records the bounded backlog that the
gate is closed against, so the deferred work is tracked rather than silently
suppressed.

## What Changes

- Close the current Tiger Style findings in the 24 affected packages.
- Where a finding is fixed, fix it: the finding is removed from the source,
  not suppressed.
- Where a finding cannot be removed without changing observable behaviour,
  changing a published public API, or restructuring code that is out of this
  change's scope, record it as a file-scoped `#![allow(tigerstyle::...)]`
  with the `HARDENING-BACKLOG` comment naming this change's evidence file
  and the standalone hardening pass it is scheduled for. The allow is scoped
  to exactly the lint categories present in that file at recording time.
- Record every deferred finding in `evidence/tigerstyle-backlog-2026-10-22.log`
  with its file, line, category and reason.
- Record the green gate transcript in `evidence/tigerstyle-final-green.txt`.

## Non-goals

- This change does not weaken `dylint.toml`. The consumer-owned lint levels
  stay exactly as they are.
- This change does not add a general `#[allow]` to any file. Only the
  `tigerstyle::` categories named in this change's evidence are scoped.
- This change does not extend any existing `HARDENING-BACKLOG` block beyond
  the categories already recorded in it.
- This change does not claim that the deferred findings are fixed, that the
  crates are release eligible, or that a green Tiger Style gate is a
  correctness claim about any adapter.
