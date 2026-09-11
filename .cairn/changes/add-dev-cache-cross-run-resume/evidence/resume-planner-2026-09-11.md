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
