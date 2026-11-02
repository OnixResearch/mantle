# Task-to-evidence map (2026-11-01)

Scope: state what each implementation task in `tasks.md` has evidenced and what
remains. No task is checked here; several obligations are only partly met.

| Task | Status | Evidence / missing |
| --- | --- | --- |
| S1 baseline | evidenced | `2026-10-31-baseline-behavior.md` records source-level facts; `2026-11-02-pool-baseline-transcript.md` captures a runtime local build with the exact binary BLAKE3, store prefix, scheduler policy, derivation, output path, and output BLAKE3. |
| S2 pool schema and admission | evidenced | `2026-10-31-pool-plan-slice.md`, `…-pool-admission-slice.md`, `…-pool-preflight-slice.md`; config, per-endpoint FD, binding, key, resource, and session admission are exercised. |
| S3 prepared envelope | evidenced | `2026-10-31-prepared-pool-goal-slice.md`, `…-remote-nix-derivation-payload-slice.md`, `…-content-addressed-leaf-slice.md` (resolved action identity). |
| S4 executor adapter | evidenced | `2026-10-31-pool-goal-executor-slice.md`, `…-goal-end-to-end-slice.md`, `…-content-addressed-leaf-slice.md`. |
| S5 Worker wiring | evidenced | `2026-10-31-worker-pool-routing-slice.md`, `…-pool-build-service-slice.md`, `…-diamond-slice.md`. |
| S6 reservations and bounds | evidenced | Revision-conflict reload/reapply, busy queueing, the global `--jobs` bound, and the pool session `max_concurrency` bound are covered (`…-diamond-slice.md`, `…-capacity-queueing-slice.md`, `…-global-jobs-slice.md`, `…-session-concurrency-slice.md`); `--remote-resource-requirements` now applies to pool goals, placement subtracts live leases from the declared inventory, an endpoint without an inventory cannot prove quantified capacity, and `production_pool_enforces_declared_resource_quantities` proves both the fitting dispatch and the oversized refusal (plus the unbound-flag refusal). |
| S7 cancellation, teardown, retries, fallback | partial | Cancellation with owned transport teardown and attempt settlement are covered (`…-cancellation-slice.md`, `…-transport-death-slice.md`); admitted pre-dispatch local fallback is implemented and tested (`…-local-fallback-slice.md`); a SIGINT during the fallback goal now reaches the sandbox and stops the local goal before its builder ends (`production_pool_local_fallback_cancellation_stops_the_local_goal`); stale-result refusal relies on existing coordinator tests. |
| S8 verified outputs, plan/source re-admission, observations | partial | Output verification/import, duplicate-completion refusal, and a returned `mantle-plan-v1` chain (two dependent units, builder source input transferred, `unit.main` bound to `unit.dep` output, chain built through the pool, output content checked) are covered; larger plans and per-goal placement/attempt/trust observations remain limited to the existing report fields. |
| S9 diamond, overlap, counts, waiters | evidenced | `…-diamond-slice.md`, `…-overlap-slice.md` prove exact counts and overlapping worker lifetimes; `…-waiter-notifications-slice.md` asserts one notification per dependent in the diamond and chain. |
| S10 concurrency, saturation, worker death, cancellation, stale fences | evidenced | Concurrent completions, queueing, session/global bounds, mid-exchange worker death, and cancellation are covered by the pool suites. `production_pool_rejects_a_stale_completion_after_reassignment` reassigns the job to fence 2 between admission and completion (debug-only seam), and the stale completion is rejected before any output is exported; the coordinator state retains the successor attempt. Stale-fence, unknown-future-fence, and lease overcommit/shrink rules remain covered by the coordinator and resource tests the executor uses. |
| S11 positive and negative admission | partial | Full-key/NAR import, untrusted key name, tampered NAR bytes, missing source input, revoked binding, ambiguous configured keys, a malformed ticket credential rejected before dispatch (worker store stays empty), a live source input produced by an earlier local build and uploaded from the client store backend, and the executor's missing/extra/wrong-output guard are covered. |
| S12 CLI compatibility scenarios | partial | Plan-only pool validation, unsupported-mode rejection, local/single-builder/pool byte parity, the client-cache-hit test, the 30-test pool suite, and the green ordinary gate cover the scenarios; `2026-11-02-pool-parity-report.md` retains the comparison digests. |
| S13 verification and delivery gates | evidenced | Ordinary gate exit 0 (226 suites), 27-test pool suite, strict Clippy, fmt, and the operator contract check. |
| S14 documentation | evidenced | `docs/remote-pool.md`, README link, and operator contract regeneration. |
| S15 validation and acceptance | blocked | Cairn validation passes (`2026-11-01-pool-cairn-validation.md`), but sync/archive wait on `resolve-remote-entry-points-through-one-binding` and `separate-remote-build-hexagon`; no acceptance yet. |

## Next obligations, in dependency order

1. Archive after `resolve-remote-entry-points-through-one-binding` and `separate-remote-build-hexagon` close (S15).
