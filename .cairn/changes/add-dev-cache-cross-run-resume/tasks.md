## Phase 1: Resume core and shell

- [x] [depends:dev-cache-source-built-fixed-point] [serial] I1 Record the current marker, provider-adoption, persistent-store, and staging-directory behavior before implementation. r[source_built_fixed_point_improved_iteration.dev_cross_run_resume]
  - Evidence: baseline recorded in `evidence/resume-planner-2026-09-11.md` (stage markers in `<staging>/.stage-markers/`, receipt-bound provider cache keyed by source authority plus five policy digests, dev store snapshot and adopted marker, per-attempt staging directories, and the absent resume planner/bundle/reporting gap).
- [x] [serial] I2 Add a pure bounded resume planner that revalidates source, plan, policy, stage, producer, and output identities. r[source_built_fixed_point_improved_iteration.dev_cross_run_resume]
  - Evidence: `src/source_built_fixed_point_resume.rs` implements the pure bounded resume planner with `StageBundleReference`, `plan_stage_resume`, `ResumePlan` (restored and executed stages reported separately), and `policy_cohort_digest`; source, plan, policy, stage, producer, and output identities are all revalidated. 7 fixtures pass (`evidence/resume-planner-2026-09-11.md`).
- [x] [serial] I3 Publish and restore content-addressed stage bundles that include the required transition execution tree and stage outputs. r[source_built_fixed_point_improved_iteration.dev_cross_run_resume]
  Evidence: `evidence/stage-bundle-resume-2026-09-11.md` (`publish_stage_bundle`/`read_stage_bundles`/`restore_stage_bundle` with recomputed digest checks; transition execution tree and outputs stored under one content address).
- [x] [serial] I4 Continue from the first incomplete stage in a fresh staging directory and report restored and executed stages separately. r[source_built_fixed_point_improved_iteration.dev_cross_run_resume]
  Evidence: `evidence/stage-bundle-resume-2026-09-11.md` (restore into the fresh staging directory plus replay-identity stage marker; `run_attempt` emits the restored/executed split; marker digest mismatch defect fixed).

## Phase 2: Verification

- [x] [parallel] V1 Add positive fresh-directory resume tests and negative tests for stale, modified, partial, unknown, mismatched, or promoted-path bundles. r[source_built_fixed_point_improved_iteration.dev_cross_run_resume]
  - Evidence: positive fresh-directory resume and negative stale, modified, partial, unknown, mismatched, duplicate, promoted-path, and missing-bundle fixtures pass (`cargo test -p mantle --bin mantle source_built_fixed_point_resume::`: 7 passed).
- [ ] [depends:prove-source-built-mantle-fixed-point] [serial] V2 Run a complete cold-to-cached-to-adopt dev cycle and record exact source, plan, provider, store, stage, report, and alias evidence. r[source_built_fixed_point_improved_iteration.dev_resume_runtime_confirmation]
- [ ] [depends:prove-source-built-mantle-fixed-point] [serial] V3 Run a fresh promoted cold proof and make sure that it starts from empty authority, ignores dev state, and emits no cache-adoption disposition. r[source_built_fixed_point_improved_iteration.dev_resume_runtime_confirmation]
- [x] [serial] V4 Run focused tests, formatting, Clippy, `git diff --check`, Cairn validation, Tracey coverage, all three gates, and relevant Nix checks. r[source_built_fixed_point_improved_iteration.dev_resume_runtime_confirmation]
  Evidence: `evidence/v4-rails-2026-09-11.md` (67 focused tests pass; fmt check exit 0; `git diff --check` exit 0; Cairn validate `valid: True findings: 0`; Tracey coverage 155/155; all three gates PASS; Tiger Style check exit 0).

## Current runtime handoff (2026-10-23)

No dev cycle, proof, build, test, formatter, or gate was run by this slice;
historical checked evidence above is not fresh acceptance. The existing
bounded planner/stage-bundle/store-adoption paths remain a development lane.
Cold source preparation rejects all dev modes, and the promoted cold route
does not use restored stage bundles or adopted stores. Current resource-bound
validation rejects invalid requests before hydration, including development
attempts. See `docs/operator-proof-guide.md` for the separate cold command and
its independently reviewed prepared-profile identity.

V2 still requires the integration owner to execute the actual complete
cold-to-cached-to-adopt cycle with the same final source/policy cohort, retain
restored/executed stage dispositions and alias evidence, and prove rejection
of stale/tampered stage bundles. V3 requires a separate fresh cold authority
and store with no dev flags or inherited compiler-cache/scheduling authority.
A reported prior success, an adoption marker, or a cached provider must never
stand in for that new v2 receipt, action reconciliation, or stage equality.

## Verified input supply, not runtime acceptance (2026-10-27)

`evidence/cold-input-supply-review-2026-10-27.json` binds the transferred,
exact-lock vendor cohort and source-input verification. The supply receipt
explicitly excludes a final source freeze, complete source-built profile,
native/StageX closure union and independent profile/provider/program admission.
Those prerequisites remain required before execution. No source preparation,
development cycle or promoted cold proof is recorded here; V2/V3 stay open.


