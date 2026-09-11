# Resume planner and fixtures (I2, V1, 2026-09-11)

Task-ID: mantle.source_built_fixed_point_improved_iteration.dev_cross_run_resume
Covers: dev_cross_run_resume

## What landed

`src/source_built_fixed_point_resume.rs` — pure resume planning over the
existing dev-cache cohort:

- `StageBundleReference` records the published content address, the
  recomputed on-disk address, plan and source-authority digests, the producer
  identity, the policy-cohort digest, the required outputs, and any promoted
  alias observed beside the bundle.
- `plan_stage_resume` walks the plan's stage order and restores a stage only
  while every earlier bundle revalidates; the first invalid or missing stage
  and every later stage execute. `ResumePlan` reports `restored_stages` and
  `executed_stages` separately with ordered typed blockers.
- Rejections: unsupported schema, stale plan, stale source authority, stale
  policy cohort (digest computed by `policy_cohort_digest`), producer
  mismatch, promoted path, malformed or mutated bundle digest, missing or
  malformed required outputs, output bound, missing bundle, duplicate bundle,
  unknown stage, and stage bound.

## Fixtures (V1)

`cargo test -p mantle --bin mantle source_built_fixed_point_resume::` →
`test result: ok. 7 passed`. Positive: a fresh directory restores every
validated stage. Negative: first-gap execution, stale plan/source/policy,
mutated/partial/malformed bundles, promoted paths, unknown stages, duplicate
bundles, and producer mismatch. The neighbouring dev-cache suite still passes
(58 tests in the `source_built_fixed_point` filter).

## Rails

`cargo fmt`, first-party Clippy (no first-party findings), `git diff --check`,
`cairn validate` (`valid: true`), Tracey coverage (`155/155`), and all three
change gates pass. `nix flake check` and the long runtime-confirmation
proofs remain.

## Open

I3 (publish and restore content-addressed stage bundles, including the
transition execution tree), I4 (continue from the first incomplete stage in a
fresh staging directory), V2/V3 (proof-dependent runtime confirmation), and
the Nix-check portion of V4.

## Baseline before implementation (I1)

Observed behavior at revision `56e7cb39b`:

- **Stage markers**: `src/source_built_fixed_point_shell.rs` writes one
  marker per completed stage into `<staging>/.stage-markers/<stage_id>`
  (`STAGE_MARKERS_SUBDIR`, `write_stage_marker`, `read_stage_marker`,
  `transition_marker_is_trusted`), validated by
  `source_built_fixed_point_dev_cache::validate_stage_marker` against the
  plan digest, stage id, and a fresh replay digest.
- **Provider adoption**: `src/source_built_fixed_point_dev_cache.rs` keys a
  receipt-bound provider entry by
  `dev_provider_cache_key(source_authority, policies)`, and
  `evaluate_provider_cache_lookup` treats any missing, stale, or mismatched
  schema/plan/key/source/policy/producer field as a hard miss
  (`write_dev_provider_cache` persists it in the shell).
- **Persistent store state**: `seed_dev_store_snapshot` plus
  `write_dev_adopted_marker` carry dev state across runs; the fast-fail check
  (`dev_fast_fail_check`, `last_published_source_digest`) can report a prior
  published success instead of rebuilding.
- **Staging directories**: each attempt materializes into its own staging
  directory; there is no content-addressed bundle for a completed stage, so a
  fresh staging directory cannot reuse earlier stage outputs and must redo
  them.
- **Gap this change closes**: no resume planner, no published stage bundles
  (transition execution tree and stage outputs), and no separate
  restored-versus-executed reporting. V2/V3 runtime confirmation depends on
  `prove-source-built-mantle-fixed-point`.
