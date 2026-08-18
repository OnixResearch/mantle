## Context

Cairn's current coverage command scans requirements under `cairn/specs/` and implementation references under `crates/` and `tools/`. Mantle's root package implementation lives in top-level `src/` and `tests/`, so a requirement can be implemented and verified without being visible to the coverage rail. A single bridge file currently covers only `rust_package_planning.source_built_toolchain_closure`.

## Approach

1. Capture a baseline JSON report from `cairn tracey coverage --root . --json` and group missing IDs by spec and feature area.
2. Decide for each group whether coverage should be a direct source/test marker, a temporary bridge marker with an evidence pointer, or an explicit debt entry.
3. Prefer direct markers near functional-core logic and focused tests when those files are scanned or can be made scanned.
4. Use bridge refs only when scanner limits prevent direct root-package refs; each bridge entry must cite implementation paths and durable evidence.
5. Keep global coverage claims bounded. If global coverage remains red, evidence must report exact missing counts and the next group to backfill.

## Risks

- Comment-only bridge refs can overstate implementation if they do not cite inspected evidence.
- Backfilling all 201 IDs in one pass is high risk; use batches with focused evidence.
- Updating scanner globs in Cairn rather than Mantle may require a separate upstream Cairn change.

## Validation

- `cairn tracey coverage --root . --json` shows either `valid: true` or a reduced, explicitly recorded missing list.
- `cairn validate --root .` remains valid.
- Focused tests pass for any implementation files touched beyond comments.
