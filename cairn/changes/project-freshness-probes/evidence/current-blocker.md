# Current Blocker — Project freshness probes

Date: 2026-07-01

## Question

Can `project-freshness-probes` be honestly drained from the current tree?

## Inspected evidence

- `crates/crunch-project-core/src/freshness.rs` now provides the pure freshness schema slice: versioned probe definitions, normalized observations, BLAKE3 value digests, bounded diagnostics, stale/unchanged/failed/skipped/network-required classification, no-network rejection for observed network probes, and bounded template rendering.
- `cairn/changes/project-freshness-probes/tasks.md` now marks the contract, pure-core implementation, pure positive tests, pure negative tests, and gate-validation tasks complete.
- Focused validation passed with `cargo test -p crunch-project-core`, `cargo check -p crunch-project-core --target wasm32-unknown-unknown`, `cairn validate --root .`, and the `project-freshness-probes` proposal/design/tasks gates.
- The remaining unchecked tasks require shell adapters for Git, HTTP text/JSON, local file/directory, and bounded command probes, plus threading observations through `mantle list-stale`, `mantle refresh`, lock updates, generated input updates, and shell/CLI fixture tests.

## Decision

Blocked as a complete drain, but no longer blocked on the pure observation/classification/template core. The active blocker is now the imperative shell and project-command integration layer; claiming complete support would still overclaim because no probe execution adapters or refresh/list-stale wiring exist yet.

## Owner

Mantle project workflow owner for the freshness adapter shell and refresh/list-stale integration.

## Next action

1. add shell adapters incrementally, starting with local file/directory probes and deterministic fixture tests;
2. add a local Git reference probe fixture before HTTP or command probes;
3. thread normalized observations into `mantle list-stale` and `mantle refresh` without mutating on list-stale;
4. preserve explicit no-network behavior before enabling dependent VCS/input workflow claims.
