# Resolution Checkpoint — Project input retention roots

Date: 2026-07-01

## Question

Can `project-input-retention-roots` be honestly drained from the current tree?

## Inspected evidence

- `cairn/changes/project-input-retention-roots/tasks.md` has all contract, implementation, and verification tasks checked.
- `cairn/changes/project-input-retention-roots/evidence/implementation-validation-2026-07-01.md` records focused positive and negative implementation validation for the retention core, project adapter, CLI persistence, diagnostics, and Cairn gates.
- Same-turn validation reran `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` followed by `nix run path:/home/brittonr/git/cairn#cairn -- gate tasks project-input-retention-roots --root .`; the tasks gate returned `valid = true` and `verdict = PASS`.
- The implementation now provides a dedicated retention ledger, root marker persistence, interrupted-update quarantine, generation selection, and diagnostics for pinned, unpinned, stale-root, missing-root, and GC-eligible records.

## Decision

Resolved. The change can be archived after the accepted spec sync and post-archive validation pass.

## Owner

Mantle project/store workflow owner for any future retention extensions.

## Next action

1. archive `project-input-retention-roots` with Cairn;
2. verify the requirement text is present in the accepted `project-workflows` spec;
3. rerun Cairn validation and record the post-archive transcript;
4. commit the archive.
