# Cross-run stage bundles and fresh-staging resume

Change: `add-dev-cache-cross-run-resume`
Task-IDs: I3, I4
Covers: `source_built_fixed_point_improved_iteration.dev_cross_run_resume`
Subject revision: `drain/next-20260911` (worktree `/tmp/mantle-next`)

## Scope delivered

`src/source_built_fixed_point_resume_bundle.rs` publishes and restores one
content-addressed bundle per completed stage:

- `publish_stage_bundle` hashes the payload tree (the StageX transition
  execution tree with its report and stage outputs) with the shared
  `release_tree_copy::hash_directory_tree` helper, stores it under
  `<root>/<bundle-digest>/payload`, and writes `bundle.json` carrying the
  plan digest, stage id, source authority digest, producer identity digest,
  policy-cohort digest, recorded content address, required output roles, and
  an observed promoted alias.
- `read_stage_bundles` recomputes every payload digest from bytes, so a
  bundle mutated after publication reports a different `recomputed_digest_blake3`
  (`missing-payload` when the payload is gone) and is never planned as
  restored.
- `restore_stage_bundle` re-hashes the payload before copying it, refuses an
  existing destination, and fails closed on drift.

`src/source_built_fixed_point_shell.rs` wires this into the dev-only path:

- `plan_dev_resume` reads `output_dir/.resume-bundles`, derives stage ids
  from the fixed-point plan, and calls `plan_stage_resume`.
- `run_attempt` emits the restored/executed split as JSON (`--json`) or a
  progress line before any stage runs.
- When `--dev-resume` is set and the fresh staging directory has no
  transition execution tree, the planned bundle is restored into that fresh
  directory and the stage marker is written from the replay identity, so the
  existing marker-trust check accepts the restored tree and skips execution.
- A freshly executed transition publishes its bundle after the marker write,
  so the next run in a new staging directory can restore it.

## Defect found and fixed

The fresh transition path wrote the stage marker with the transition *report*
digest while `transition_marker_is_trusted` re-derives the transition from its
primary inputs (`transition_input_replay_digest`). The marker could therefore
never validate, so `--dev-resume` always re-executed the transition. The
marker now binds the replay identity, which is what the trust check compares.

## Evidence

```
nix develop -c cargo test -p mantle --bin mantle source_built_fixed_point_resume
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 2412 filtered out; finished in 0.01s

nix develop -c cargo test -p mantle --bin mantle source_built_fixed_point_dev_cache
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2422 filtered out; finished in 0.00s
```

The bundle tests cover the positive round trip (publish, read, restore into a
fresh directory) and the negative space: tampered payload, missing payload,
existing restore destination, missing payload directory at publish time, a
bundle recorded under another plan (executed, not restored), and idempotent
republication that reuses the stored bundle instead of rewriting it.

## Machine-contract coverage interaction

`scripts/check-machine-schema-contracts.rs` rejects any `src/**` file that
contains a root JSON serialization token without an inventory family decision.
The first draft of the bundle module serialized `bundle.json` itself and was
reported as one more uncovered producer. The module now writes the reference
through the shared `write_json_create_new` writer used by the stage markers and
reuses an existing reference instead of rewriting it, so the change adds no new
root JSON producer.

```
$ nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs
45 findings, none of them from this change (the pre-existing uncovered sources
remain: src/source_built_fixed_point_shell.rs, src/operator_contract.rs, ...)
$ nix develop -c cargo test -p mantle --bin mantle source_built_fixed_point_resume
test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 2412 filtered out; finished in 0.01s
```

## Non-claims

- Only the StageX transition stage publishes a bundle today; later stages are
  always executed. `plan_stage_resume` reports those as `resume-missing-bundle`
  blockers while still listing them under `executed_stages`.
- The end-to-end proof claim (a resumed run reaching the fixed point in a
  fresh staging directory) remains with the pending proof task V2/V3 and is
  not claimed here.
- No promoted-path behavior changed: every hook is gated on `--dev-resume`.
