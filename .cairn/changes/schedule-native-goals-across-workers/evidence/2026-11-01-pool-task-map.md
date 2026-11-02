# Task-to-evidence map (2026-11-01)

Scope: state what each implementation task in `tasks.md` has evidenced and what
remains. No task is checked here; several obligations are only partly met.

| Task | Status | Evidence / missing |
| --- | --- | --- |
| S1 baseline | partial | `2026-10-31-baseline-behavior.md` records source-level facts only; no runtime reproduction with exact binary/config identities and no admitted single-builder/dynamic-plan baseline. |
| S2 pool schema and admission | evidenced | `2026-10-31-pool-plan-slice.md`, `…-pool-admission-slice.md`, `…-pool-preflight-slice.md`; config, per-endpoint FD, binding, key, resource, and session admission are exercised. |
| S3 prepared envelope | evidenced | `2026-10-31-prepared-pool-goal-slice.md`, `…-remote-nix-derivation-payload-slice.md`, `…-content-addressed-leaf-slice.md` (resolved action identity). |
| S4 executor adapter | evidenced | `2026-10-31-pool-goal-executor-slice.md`, `…-goal-end-to-end-slice.md`, `…-content-addressed-leaf-slice.md`. |
| S5 Worker wiring | evidenced | `2026-10-31-worker-pool-routing-slice.md`, `…-pool-build-service-slice.md`, `…-diamond-slice.md`. |
| S6 reservations and bounds | partial | Revision-conflict reload/reapply, busy queueing, the global `--jobs` bound, and the pool session `max_concurrency` bound are covered (`…-diamond-slice.md`, `…-capacity-queueing-slice.md`, `…-global-jobs-slice.md`, `…-session-concurrency-slice.md`); explicit resource quantities are still not enforced in placement (requested capacity is zero). |
| S7 cancellation, teardown, retries, fallback | partial | Cancellation with owned transport teardown and attempt settlement are covered (`…-cancellation-slice.md`, `…-transport-death-slice.md`); admitted pre-dispatch local fallback is implemented and tested (`…-local-fallback-slice.md`); stale-result refusal relies on existing coordinator tests and fallback cancellation is observed only after the local goal completes. |
| S8 verified outputs, plan/source re-admission, observations | partial | Output verification/import and duplicate-completion refusal are covered; re-admission of a returned dynamic plan/source tree is not exercised; per-goal placement/attempt/trust observations are limited to the existing report fields. |
| S9 diamond, overlap, counts, waiters | evidenced | `…-diamond-slice.md`, `…-overlap-slice.md` prove exact counts and overlapping worker lifetimes; `…-waiter-notifications-slice.md` asserts one notification per dependent in the diamond and chain. |
| S10 concurrency, saturation, worker death, cancellation, stale fences | partial | Concurrent completions, queueing, session/global bounds, mid-exchange worker death, and cancellation are covered by the pool suites. Stale-fence, unknown-future-fence, and lease overcommit/shrink rules are covered by the coordinator and resource tests the executor uses (`stale_report_is_rejected_before_output_admission`, `malformed_identity_and_unknown_future_fence_fail_without_state_change`, `lease_snapshot_rejects_corruption_and_overcommit`), but no pool-specific two-attempt scenario is run. |
| S11 positive and negative admission | partial | Full-key/NAR import, untrusted key name, tampered NAR bytes, missing source input, revoked binding, ambiguous configured keys, and the executor's missing/extra/wrong-output guard are covered; a live private-input case and malformed credential case are not. |
| S12 CLI compatibility scenarios | partial | Plan-only pool validation, unsupported-mode rejection, local/single-builder/pool byte parity, the client-cache-hit test, the 27-test pool suite, and the green ordinary gate cover the scenarios; no separately retained comparison report file exists. |
| S13 verification and delivery gates | evidenced | Ordinary gate exit 0 (226 suites), 27-test pool suite, strict Clippy, fmt, and the operator contract check. |
| S14 documentation | evidenced | `docs/remote-pool.md`, README link, and operator contract regeneration. |
| S15 validation and acceptance | blocked | Cairn validation passes (`2026-11-01-pool-cairn-validation.md`), but sync/archive wait on `resolve-remote-entry-points-through-one-binding` and `separate-remote-build-hexagon`; no acceptance yet. |

## Next obligations, in dependency order

1. Runtime baseline with exact identities (S1).
2. Quantified resource placement and a global-job/session-bound scenario (S6).
3. Admitted pre-dispatch fallback and stale-fence/lease-ownership proof (S7, S10).
4. Dynamic plan/source re-admission through a pool result (S8).
5. Waiter-notification assertion (S9).
6. Remaining negative admission cases (S11).
7. Single-builder/cache-hit comparison report (S12).
