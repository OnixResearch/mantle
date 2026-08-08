# Promote durable publication adoption

## Why

Mantle accepted durable publication at `d1f3d6d96e2b0d9cd8497cd89b8e5a93d4a7dfaf`, but canonical `main` advanced on a separate history. Current `main` already contains a newer equivalent adoption tree and evidence. The accepted commit is still not an ancestor, so canonical history does not preserve the reviewed acceptance lineage.

## What Changes

- Build one reviewed merge candidate with current canonical `main` as the first parent and the accepted adoption commit as the second parent.
- Keep the newer canonical form for duplicate files and limit first-parent changes to this promotion lifecycle package plus the adoption receipt and validator's refreshed canonical file bindings.
- Require Onix Core canonical `main` to contain reconciliation archive `bc4629c9e766d3db82e4dab9fe8c166c360b8435` and accepted admission commit `b8387cd7d59fa3b0d4ea67646352dd27c4f7d7ed`.
- Re-run focused adoption, package, formatting, product Clippy, Tiger Style, Nickel, Nix, Cairn, and traceability checks. Require focused adoption checks to pass. Record only exact pre-existing broad failures that the successor broad-validation change owns.
- Advance Mantle `main` only through an authorized normal fast-forward push.

## Impact

This change preserves accepted ancestry on the canonical branch. It does not alter publication mechanics, rollback policy, source pins, product manifests, or release authority.

## Non-claims

Canonical branch placement does not prove whole-Mantle correctness, release eligibility, future producer availability, or reboot recovery.
