# Focused validation summary

## Baseline and compatibility

- Clean `crunch-pipeline` baseline: 40 unit tests and 19 integration tests passed; 4 integration tests remained ignored.
- Final `crunch-pipeline`: 42 unit tests and 19 integration tests passed; the same 4 integration tests remain ignored.
- Final `crunch-build-planning-core`: 37 tests passed. This includes the 24 moved route tests and 13 new planning and concurrency tests.
- Mantle `realization_routing::`: 24 tests passed, equal to baseline.
- Mantle `build_plan::`: 4 tests passed and 1 remained ignored, equal to baseline.
- Mantle `build_planning_hexagon::`: 3 adapter and freshness tests passed.
- The checked route matrix covers local cache, substitution, archive, source bundle, remote builder, local build, and preflight.
- Existing route reports, reason codes, ordering, JSON, and root command behavior remain compatible.

## Planning boundaries

- The full route kernel moved to `crunch-build-planning-core`; `src/realization_routing.rs` is now a compatibility re-export plus the retained tests.
- Cache, substitution, build, and preflight action choice delegates to `select_build_action`.
- Host parallelism is observed in the pipeline shell and converted before `plan_parallelism` runs.
- Remote candidate identity comes from `crunch-remote-core::RemoteCommand` without opening a session.
- Plan-bound effects require a freshness recheck. Stale facts, wrong identities, and route substitution fail before the executor closure runs.
- The route-plan machine schema now names `crates/crunch-build-planning-core/src/lib.rs::RoutePlanReport` as its Rust owner.

## Architecture and quality

- The architecture checker reports zero findings and detects 14 forbidden authority fixtures.
- Focused Nix checks for architecture, core tests, and core `wasm32-unknown-unknown` compilation passed.
- Strict first-party Clippy passed with `-D warnings`.
- The exact Tiger Style Nix gate passed without an allowance, baseline, or reduced scope.
- Machine-contract generation and validation passed with 24 contracted and 57 classified surfaces.
- Durable-publication adoption passed with refreshed exact Cargo and flake bindings.
- Formatting and `git diff --check` passed.
- Cairn validation reports `"valid": true`.
- Tracey reports 155/155.
- Proposal, design, and tasks gates return PASS with 10 completed tasks and V5 pending committed-source validation.

## Oracle checkpoint

- **Question:** Does the extraction separate observations, route and concurrency policy, and effect execution without changing accepted behavior?
- **Inspected evidence:** The moved route suite, checked route matrix, explicit parallelism tests, pipeline and build-plan baselines, remote projection, freshness tests, architecture negatives, Clippy, Tiger Style, machine contracts, and focused Nix checks.
- **Decision:** Accept the extraction. Keep all store, filesystem, doctor, key, host, remote-session, mutation, execution, and rendering authority in the shell and adapters.
- **Owner:** Mantle maintainers.
- **Next action:** Commit the implementation, then run committed-source Nix and lifecycle validation.

## Non-claims

The evidence does not prove effect execution, remote-worker correctness, output admission, route optimality, reproducibility, or release eligibility.
