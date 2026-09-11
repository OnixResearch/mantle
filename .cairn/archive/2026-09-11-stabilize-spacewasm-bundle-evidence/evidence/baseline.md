# Baseline: SpaceWasm report instability

## Scope

This is imported historical failure evidence, not a new build or repair result. The proposal session preserves the earlier captures without modifying their bytes.

- Producer repository: Mantle.
- Producer source: `a141fcbaafe41f9a413a81275a33fe915bfca370`.
- SpaceWasm source: `e24cf09355a90497148eb5029fdb8e3400bd63e3`.
- Frozen failing consumer: ChaosControl `6509820dee785ea288be1cbdb6e87b7a0c95194e`.
- Published consumer review: ChaosControl `bf22422c925e47779a635b5344c66b8ed1039470`, `.cairn/changes/adopt-campaign-core/spacewasm-bundle-review.md`.

## Retained observation

The consumer expected manifest BLAKE3 `13058ea2d9913348a203cceff7b58d98b6446610ac80518dc3359b8d7ee57472`.

The produced manifest measured `ded66a4959c9efeda62f2eb3d13a06de6df0ad01a1d53f222c199ab6e66d9eb7`.

The runner still measured `be8aeb698afdecf6fb608910980292517ed952f122b6447705d4bdae485b0221`.

The checked derivation was:

```text
/nix/store/pqdh5xf8y0k76h7pjvaskxbq289crkwg-spacewasm-e24cf09355a90497148eb5029fdb8e3400bd63e3-unit-tests.drv
```

The recorded command used Nix check mode, not replacement of the original output:

```sh
rebuild_timeout_seconds=300
timeout "$rebuild_timeout_seconds" nix-store --realise \
  /nix/store/pqdh5xf8y0k76h7pjvaskxbq289crkwg-spacewasm-e24cf09355a90497148eb5029fdb8e3400bd63e3-unit-tests.drv \
  --check --keep-failed --option builders ''
```

The command returned failure because the rebuilt output differed. Both captured runs report the same suite success, but the raw logs differ. Compilation took 4.74 seconds in the original capture and 3.76 seconds in the rebuild. Test completion order and the reported test duration also changed.

The producer hashes those raw outputs into receipts and includes them as bundle members. This proves instability of that report-producing derivation. It does not prove that timing alone explains all differences from the unavailable historical bundle.

## Imported artifacts

- `producer-original/`: original stdout, stderr, and receipt.
- `producer-rebuilt/`: check-mode stdout, stderr, and receipt.
- `rebuild.log`: exact Nix nondeterminism diagnostic.
- `rebuild.exit`: recorded command status.
- `outputs.diff`: exact comparison of the retained directories.
- `observed-hashes.txt`: direct measurements of the manifest and runner.

The original capture source is the operator evidence directory `campaign/.pi/complete-20260906/`. These copies remove that directory as a requirement for review. Embedded local paths are historical locators, not portable dependencies or authority grants.

## Proposal-session policy baseline

Before package creation, the retained Cairn CLI rejected Mantle's repository policy:

```text
error: failed to parse policy cairn-policy/generated/cairn-policy.json: cairn.workflow.profile.registry at workflow_profile_policy.schemas: initial workflow profile is missing: outcome-machine
```

This is a pre-existing validation blocker for this CLI/policy combination. This package does not change policy or claim that alternative-policy validation establishes repository-policy acceptance.

## Owner review checkpoint

- Question: Can the consumer safely refresh its expected digest as the repair?
- Inspected evidence: the exact producer source, consumer failure, same-derivation Nix diagnostic, and paired raw reports.
- Decision: No. A producer-owned stable-fact and raw-evidence contract must precede consumer migration.
- Owner: Mantle SpaceWasm materialization maintainer, with separate ChaosControl admission review.
- Next action: review versioning and retention, then implement in an isolated worktree with baseline and negative controls.
