# Focused validation (2026-09-11)

Task-ID: mantle.realization_routing.verification
Covers: explicit_observation_boundary, explicit_parallelism_facts,
        plan_execution_separation, typed_planning_blockers

## Commands and results

Command: `nix develop -c cargo test -p crunch-pipeline`

    test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured
    test result: ok. 19 passed; 0 failed; 4 ignored; 0 measured

Command: `nix develop -c cargo test -p mantle --bin mantle realization_routing::`

    test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 2377 filtered out

Command: `nix develop -c cargo test -p mantle --bin mantle build_plan::`

    test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 2410 filtered out

Command: `git diff --check` — clean.

Command: `cairn validate --root .` — `"valid": true`; proposal/design/tasks
gates PASS after task ordering was restored.

## What this covers

- Pure route planning over explicit candidates with typed selections,
  rejections, reason codes, and named bounds; remote facts arrive through
  `RemoteBuilderPlanFacts` without sessions.
- Freshness recheck (`recheck_route_plan_freshness`) rejecting stale plans
  and policy drift instead of re-planning silently.
- Pure jobs policy (`jobs_policy.rs`) over requested jobs, observed
  parallelism, policy cap, and executor limit.
- Route matrix and negative fixtures for missing observations, offline
  network routes, simultaneous blockers, discovery order, and substitution.

## Open

V5 remains: the full rail run including `nix flake check -L`. First-party
Clippy and the machine-contract check are pre-existing red on `main`
(reproduced on a clean `af85ab857` checkout), so V5 cannot be fully green
until those are repaired by their owners.
