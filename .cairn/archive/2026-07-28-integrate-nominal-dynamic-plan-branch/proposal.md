# Proposal: Integrate the reviewed nominal dynamic-plan branch

## Why

The `replace-plan-primitive-aliases` branch now contains one reviewed and validated implementation commit.

Mantle main has advanced beyond the source branch parent. The main checkout also contains unrelated StageX work that integration must not overwrite.

Mantle needs a bounded integration change. The change must bind the exact source commit, preserve newer target work, and rerun evidence on the resolved target state.

## What Changes

- Integrate Git commit `a2c872cfe6826df2d3ca7c85597365e6076fdfba` from `replace-plan-primitive-aliases`.
- Use a dedicated clean branch and worktree from the selected main target commit.
- Resolve overlapping files as a semantic union of target work and the nominal dynamic-plan change.
- Preserve the source change archive, accepted build-correctness requirements, documentation, fixtures, and Octet configuration.
- Rerun focused, negative, compatibility, policy, quality, Nix, and Cairn checks after conflict resolution.
- Record the source commit, target commit, resolved tree, canonical digest, test results, and bounded blockers.
- Sync and archive this integration change only after the resolved target passes its required gates.

## Dependencies

- Source branch: `replace-plan-primitive-aliases`
- Source commit: `a2c872cfe6826df2d3ca7c85597365e6076fdfba`
- Source tree: `17c155109a663ff3574c4865c3c905ee0e5ee486`
- Source parent: `01dcfc6e05daf7dd56b0f483714bb695291f66ad`

The current StageX edits need their own preserved target state before integration execution begins.

## Non-Goals

- Rewriting the reviewed source commit or its archived evidence.
- Removing or changing unrelated StageX work.
- Changing the `mantle-plan-v1` wire schema or accepted canonical digest.
- Weakening private fields, checked constructors, digest roles, or typed graph storage.
- Claiming store presence, source trust, build success, compiler correctness, or release eligibility.
- Pushing, moving main, or opening a pull request without separate operator instruction.

## Impact

- **Primary files**: dynamic-plan core and wire adapters, worker integration, workspace policy metadata, documentation, fixtures, accepted specs, and lifecycle evidence.
- **Integration risk**: target changes may overlap source files or accepted specifications.
- **Validation**: focused Cargo tests, compile-fail tests, Octet, Clippy, Tiger Style, Nix checks, Cairn gates, and target-preservation review.
