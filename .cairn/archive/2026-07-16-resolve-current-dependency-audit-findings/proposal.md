## Why

Mantle's checked-in dependency audit currently fails on three actionable findings: `crossbeam-epoch 0.9.18` is affected by `RUSTSEC-2026-0204` despite a compatible fixed release, the review-approved immutable `nickel-export-core` repository is not named in source policy, and `winx`'s exact `Apache-2.0 WITH LLVM-exception` expression is not named in license policy. Leaving these mixed together as generic audit debt obscures the security fix and prevents the repository-owned audit rail from serving as release evidence.

## What Changes

- Move only `crossbeam-epoch` to the compatible fixed release and prove the vulnerable lock entry is gone without adding an advisory waiver.
- Admit only the already-specified `OnixResearch/nickel-export` repository while preserving its exact revision and independent pin checks.
- Admit only the exact SPDX license expression actually declared by `winx`; do not broaden license matching or lower confidence.
- Refresh dependency-audit documentation and positive/negative evidence so default policy, floating Git sources, and unreviewed license expressions remain rejected.

## Impact

- **Files**: `Cargo.lock`, `deny.toml`, `docs/dependency-audit.md`, and this Cairn change package.
- **Testing**: checked-policy `cargo-deny`, focused locked compile/tests for the affected dependency path, immutable Nickel export pin checks, negative policy guards, first-party quality checks, and Cairn validation/gates.

## Out of Scope

- Broad dependency upgrades unrelated to the three current failures.
- New advisory waivers for a vulnerability with a compatible fixed version.
- Wildcard Git-source or license approval.
- Changing the selected `nickel-export-core` revision or its authority boundary.
