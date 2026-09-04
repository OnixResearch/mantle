# Dev resume architecture search

Date: 2026-09-04

## Goal

Restore a validated stage prefix into a fresh dev staging directory. Keep promoted proof authority cold and unchanged.

## Inspected evidence

- `src/source_built_fixed_point_dev_cache.rs` owns the existing pure provider-cache and marker decisions.
- `src/source_built_fixed_point_shell.rs` owns source materialization, the persistent dev store, stage execution, and reports.
- `src/source_built_fixed_point_checkpoint.rs` binds the first four provider stages for promoted checkpoints.
- `src/source_built_fixed_point_checkpoint_shell.rs` owns bounded observation, copy, no-replace publication, and restore.
- `src/source_built_fixed_point_shell/checkpoint_integration.rs` reconstructs validated provider state after restore.
- `src/cargo_free_self_build.rs` owns Mantle stage 1 and stage 2 execution.
- Searches across `/home/brittonr/git/OnixResearch` found no separate reusable cross-run stage-resume component.

## Candidate routes

### Copy attempt-local markers

This route is small, but a marker does not carry required stage state. It also does not remeasure source, policy, producer, or output identities. Rejected.

### Infer completion from the persistent store

Store presence cannot prove which stage produced a path. It cannot prove transition-tree or execution-evidence linkage. Rejected.

### Reuse promoted checkpoints as dev authority

The existing checkpoint shell has useful bounded I/O. Its `PromotedExecution` origin must not become dev authority. Direct reuse would mix evidence domains. Rejected.

### Add a dedicated dev resume core and adapter

A new `no_std + alloc` core can validate stage manifests and select a contiguous restored prefix. A dev-only shell can reuse the existing bounded payload I/O patterns. Promoted mode rejects resume before cache access. Selected.

## Decision

Use a separate dev resume contract with six ordered stage kinds. Publish one content-addressed manifest for each completed stage. Share immutable payload objects by BLAKE3 identity.

The core receives only explicit facts. It validates source, plan, policy, stage, producer, output, payload, and bundle identities. It returns restored stages, executed stages, and the first incomplete stage.

The shell owns filesystem observation, object publication, no-follow copy, restore, process execution, and report writing. It must remeasure restored objects before the core can select them.

Provider payload handling can reuse existing checkpoint observation and restore helpers. The origin remains explicitly `DevExecution`. The promoted path never reads this namespace.

## Adversarial review

- A modified object changes its measured digest and rejects the candidate.
- A changed source, plan, or policy rejects the candidate before restore.
- A wrong producer or output identity rejects the candidate.
- A missing or unknown stage, payload, or manifest forces execution from an earlier boundary.
- A partial publication has no final manifest and cannot authorize a skip.
- Two conflicting candidates for one stage do not create broader authority.
- A promoted request returns a cold plan without reading cache content.

## Non-claims

Resume proves only identity-bound restoration for the selected stage prefix. It does not prove that skipped work ran in the new attempt. It does not prove compiler correctness, release eligibility, or broad reproducibility.
