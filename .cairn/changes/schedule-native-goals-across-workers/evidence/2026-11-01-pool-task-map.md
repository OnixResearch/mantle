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
| S6 reservations and bounds | partial | Revision-conflict reload/reapply and busy queueing are covered (`…-diamond-slice.md`, `…-capacity-queueing-slice.md`); explicit resource quantities are not enforced in placement (requested capacity is zero) and no global-job/session-bound scenario is exercised. |
| S7 cancellation, teardown, retries, fallback | partial | Cancellation with owned transport teardown and attempt settlement are covered (`…-cancellation-slice.md`, `…-transport-death-slice.md`); stale-result refusal relies on existing coordinator tests; admitted pre-dispatch local fallback is not implemented or exercised. |
| S8 verified outputs, plan/source re-admission, observations | partial | Output verification/import and duplicate-completion refusal are covered; re-admission of a returned dynamic plan/source tree is not exercised; per-goal placement/attempt/trust observations are limited to the existing report fields. |
| S9 diamond, overlap, counts, waiters | partial | `…-diamond-slice.md`, `…-overlap-slice.md` prove exact counts and overlapping worker lifetimes; one waiter notification per goal is not asserted. |
| S10 concurrency, saturation, worker death, cancellation, stale fences | partial | Concurrent completions, queueing, mid-exchange worker death, and cancellation are covered; stale-fence replacement, capacity shrink, and lease-ownership proof are not separately exercised. |
| S11 positive and negative admission | partial | Full-key/NAR import, untrusted key name, tampered NAR bytes, missing source input, and revoked binding are covered; same-name-different-material, missing/extra output, private input, and malformed credential cases are not. |
| S12 CLI compatibility scenarios | partial | Plan-only pool validation, unsupported-mode rejection, the 21-test pool suite, and the green ordinary gate cover local/single-builder/cache-hit behavior; a dedicated single-builder/cache-hit comparison report was not produced. |
| S13 verification and delivery gates | evidenced | Ordinary gate exit 0 (226 suites), focused pool suites, strict Clippy, fmt, and the operator contract check. |
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
