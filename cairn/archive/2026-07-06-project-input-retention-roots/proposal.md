## Why

The `project-workflows` spec already accepts
`[depends:project_workflows.input_retention_roots]` and
`[depends:project_workflows.input_retention_atomicity]`: Mantle MUST support a
project-level default `retention` policy and per-input overrides
(`untracked`, `current`, `recent-generations` with a bounded positive integer),
MUST distinguish pinned, unpinned, stale-root, missing-root, and
garbage-collection-eligible records, and MUST update retention roots atomically
with lock/source-state transitions into `.mantle/retention.json` and
`.mantle/retention-roots/`.

A targeted grep found no retention-root materialization in
`crates/crunch-project/src`, so this is the largest genuine implementation gap
among the project-workflows scaffolds. This change implements and proves the
retention capability with a bounded offline proof rail and versioned,
non-overclaiming evidence.

## What Changes

- Implement project input retention-root materialization and diagnosis in
  `crates/crunch-project` and the project command shell: current, recent
  generations, and untracked modes; pinned/unpinned/stale-root/missing-root/
  garbage-collection-eligible classification; named, bounded, validated
  generation limits; atomic commits via same-directory temp files.
- Provide a bounded local offline proof rail that exercises the three retention
  modes through refresh/import/generated-input updates and `mantle check`.
- Emit a versioned, redacted, non-overclaiming evidence record that classifies
  each input retention state and binds the input name, lock digest, source
  identity, and content digest.
- Add negative cases: interrupted root update is not durable; stale-root and
  missing-root are diagnosed; untracked input is garbage-collection-eligible and
  not reported as pinned or durable; generation selection is not based on
  filesystem timestamp ordering.

## Impact

- **Files**: new `crates/crunch-project/src/retention.rs` (plus adapter wiring
  in `lib.rs`), `src/project_cmd.rs`, `src/project_resolve.rs`, a bounded offline
  proof rail, and an evidence-render helper.
- **Testing**: positive composition proof for the three modes, interrupted-update
  and stale/missing-root negative cases, untracked-gc-eligible assertion,
  generation-limit validation, atomicity assertion, and the Cairn gates.

## Out of Scope

- Store-level garbage collection implementation (already owned by `crunch-store`).
- Treating retention roots as build correctness or release reproducibility proof.
- Changes to the accepted `input_retention_roots` or `input_retention_atomicity`
  requirement text.
