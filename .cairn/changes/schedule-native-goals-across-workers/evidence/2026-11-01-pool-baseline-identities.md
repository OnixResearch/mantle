# Pool baseline identities (2026-11-01)

Scope: record the exact tree, binary, and runtime paths exercised for the pool
change. This is a baseline record, not a new acceptance claim.

## Identities

| Item | Value |
| --- | --- |
| Source tree | `f3f8fdf47017c3e9f1a1f44080b3c27250ce8f63` |
| Debug `mantle` binary | `/var/tmp/mantle-test-target/debug/mantle` |
| Binary size | 1,875,825,448 bytes |
| Binary BLAKE3 | `bcfcf5ca37e543a41be384b519ecdfd49b52f6053b5c1b38dcba035b4180a9b1` |
| Store prefix | `/mantle/store` |
| Pool configs | single endpoint/session 1; two endpoints/session 2 with declared inventory; test-only dead, tampered, and worker-killing wrappers |

The binary carries debug info; the pool tests use a small exec wrapper as the
transport program because the transport-program bound rejects the 1.8 GiB
binary.

## Runtime paths exercised (all on the identity above)

| Path | Test |
| --- | --- |
| local build of the replay fixture | `production_pool_output_matches_local_build` |
| single-builder remote build | `production_single_builder_and_pool_outputs_match`, existing `production_stdio_*` suites |
| pool single goal | `production_pool_dispatches_one_goal_through_the_worker` |
| CA pool goal | `production_pool_dispatches_one_content_addressed_goal_through_the_worker` |
| two-worker diamond and overlap | `production_pool_dispatches_a_diamond_across_two_workers`, `production_pool_diamond_worker_lifetimes_overlap` |
| CA diamond | `production_pool_dispatches_a_content_addressed_diamond_across_two_workers` |
| global `--jobs` and session bounds | `production_pool_respects_the_global_job_limit`, `production_pool_respects_the_session_concurrency_limit` |
| cancellation, worker death, tampering, admission failures | the failure-settlement test phases |
| plan-only pool validation | `pool_plan_validates_selection_without_tickets` |

## Missing from this baseline

- A runtime baseline of the dynamic-plan path through a pool goal; local
  dynamic-plan tests exist, the pool variant does not.
- A signed-cache-hit comparison between the single-builder and pool paths.
- Any baseline on a second host or non-loopback transport.
