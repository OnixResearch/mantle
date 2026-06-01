## Why

Mantle's canonical specs now include 202 Tracey requirement IDs, but `cairn tracey coverage --root . --json` reports only one referenced requirement and 201 missing references. The source-built toolchain closure archive added a narrow bridge for one requirement, but the release-readiness rail remains globally red. Review can no longer distinguish implemented-but-unannotated requirements from genuinely unimplemented requirements.

## What Changes

- Add a maintained coverage-readiness workflow for accepted specs.
- Backfill implementation and verification references for accepted requirements in bounded batches.
- Replace ad hoc bridge comments with evidence-backed references or a deterministic debt ledger for requirements that cannot yet be honestly linked.
- Preserve proof-before-claim behavior: coverage backfill may prove traceability only, not feature support beyond existing evidence.

## Impact

- **Files**: `tools/tracey_refs.rs`, root package/source comments, archived evidence links, and possibly Cairn Tracey coverage configuration if the scanner needs root-package globs.
- **Testing**: `cairn tracey coverage --root . --json`, `cairn validate --root .`, focused tests for any touched implementation modules, and an evidence transcript that records remaining missing counts or full pass.
