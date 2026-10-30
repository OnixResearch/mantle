# T4.3 retrospective waiver and current-tree after observation, 2026-10-30

Task T4.3 (`tasks.md`): "Run the demand, retraction, deduplication, and no-inference rails
before and after the change. Preserve exact results." Requirement
`r[build_scheduling.demand_without_capacity_inference]`.

## Decision

The operator authorized accepting T4.3 on its after side with the before side waived
([operator authorization](../../reconcile-main-api-lineages-2026-10-17/evidence/operator-authorization-2026-10-30.md), row 6). This file records the basis. It changes nothing
that was observed, and it claims no historical pass.

## Why the before side cannot be run as written

`before-after-observations-2026-10-27.json` records that the historical coordinator
(commit `dbeb354ba29095467e13ee615f229d216e4e5fb6`) has no demand publication, retraction or
deduplication capability, so those rails are absent capabilities, not passing tests. The only
historical probe was a no-inventory admission on a source instrumented with one added test
body, and its inputs (policy, ingress and inventory) differ from the current gateway fixture,
so no identical-vector comparison exists. None is claimed.

## After side, executed on the current tree

Subject: commit `1776617592987af585e48e708961c962cde3407e`, tree `2e907c0c611080595e0bb882a4b430e7a9732ab4`. The coordinator and core rails ran while eight documentation files (ADR status text, three `tasks.md` files, `adr/README.md` and the authorization record) were uncommitted; no source file differed from the commit. The real-daemon rail started at that commit; a documentation-only commit (`3fd6149b0`) landed during it, `git status --porcelain` was empty at its end, and the harness reported the 13 selected source files unchanged across the run.

| Rail | Command | Result (verbatim) |
|---|---|---|
| Coordinator | `cargo test --locked --offline -p mantle --bin mantle worker_demand -- --test-threads 1` | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2842 filtered out; finished in 0.91s` |
| Core | `cargo test --locked --offline -p crunch-remote-core demand -- --test-threads 1` | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 105 filtered out; finished in 0.01s` |
| Real daemon | `cargo test --locked --offline -p mantle --test demand_runtime -- --ignored --nocapture --test-threads=1` | `test real_gateway_derived_demand_and_ordinary_reports_keep_quantified_blocker ... ok` then `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 435.51s` |

The real-daemon rail starts a long-lived production gateway, daemon and subscriber. Its result
(`demand-runtime-current-tree-2026-10-30.result.json`, BLAKE3 `7bd3c86f9fab8181e19a632ed1532cfd05ab5ce2e33d3b959cabc36011b48ee9`) records scenario
`actual-long-lived-production-gateway-derived-demand`, status `passed`: two request lifetimes, each published then
retracted, blocker `resource-memory-unavailable` (declared 67108864 bytes
against 33554432 admitted), `jobs` 0, `resource_leases`
0, and identical demand facts from plan and build with the daemon absent
and present. The test binary's BLAKE3 was `dc5cd6cf60debdc060da5a932351c9aac92f5b075d012505919d420eb6506cc3`.

Raw outputs: `retrospective-waiver-2026-10-30-captures.json` (BLAKE3 `a567bddf32725eda208045a5716f57e55c0bec7fd8d3b95280b79b192404336f`) retains the complete result of both runs, with the exact stdout and stderr and a BLAKE3 for each stream. Absolute paths inside it name the original run directory.

### First real-daemon launch failed on harness input

The coordinator and core rails ran in a command set that also launched the real-daemon rail (`decision-evidence-v93-demand`, step `demand-runtime-actual`). That launch exited 101 after 0.00 s: the test panicked at `tests/demand_runtime.rs:178` with `set a fresh retained runtime root`, because the fixture environment (`MANTLE_DEMAND_RUNTIME_ROOT`, `MANTLE_RESOURCE_INPUT_PATHS`) was not passed. No scenario ran, so it is a harness-input failure, not a product failure, and it is kept as recorded. The driver for that set exited 1 (the coordinator and core steps exited 0). The real-daemon row above is from a separate relaunch (`decision-evidence-v94-demand-runtime`) with the fixture environment set; its driver exited 0.

## Fixture regeneration

The 2026-10-27 run's signed input descriptor no longer existed. It was regenerated for this
run: a fresh fixture signing key (`mantle-demand-fixture-20261030-1`), a local file cache
holding the static BusyBox store path the earlier run used, and the public key as the only
trusted key. The private key stays mode 0600 in the run directory and is not evidence.

## Differences from the 2026-10-27 run

8 of 17 result fields differ: `absent_build_demand`, `absent_plan_demand`, `binary`, `live_socket`, `present_build_demand`, `present_plan_demand`, `rounds`, `source_drv_path`. In each of the four
demand objects, only three digests differ: `job_id`, `requirement.placement_key_blake3` and
`requirement_id_blake3`, all of which follow the derivation; the blocker reason, policy id and
resource quantities are identical. The other differences are the derivation path, the live
socket path, the round contents and the binary identity. Of the 13 source files
that run recorded, 3 have changed since: `src/main.rs`, `Cargo.lock`, `src/nix_gateway/api_host.rs`. The
`api_host.rs` change is commit `bd22956ec`, which added operator-declared `required_resources`
per system to the gateway configuration. This run is the observation that covers it.

## Scope

Accepted: T4.3, as an absent-before / passing-after capability comparison with the after
side observed on the current tree.

Not claimed (from the result's own non-claims and this file): no capacity inference, no
supervisor provisioning, no blocked work executed, no reliable production delivery, no
demand-created capacity, lease or reservation authority, and no repeated pending placement
rounds (the two lifetimes are repeated fail-fast requests). T4.5 (spec sync and archive) is a
separate lifecycle step.
