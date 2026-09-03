# 0114: Plan build routes from explicit observations

- Status: Accepted
- Date: 2026-09-03

## Context

Mantle already had a deterministic realization-routing module. However, that module lived in the CLI crate. `src/build_plan.rs` still chose cache, substitution, build, and preflight actions after store and doctor probes.

`crunch-pipeline::resolve_max_jobs` also called host available-parallelism inside its clamp policy. This mixed a nondeterministic observation with a deterministic scheduling decision.

`crunch-composition-core` owns castore composition, and `crunch-remote-core` owns remote protocol decisions. Neither component owns build-route preference or local concurrency policy.

## Decision

Promote the existing realization-routing kernel intact into `crunch-build-planning-core`. Keep `src/realization_routing.rs` as the stable compatibility re-export and retain its test matrix.

Extend the core with bounded explicit observations for local output, source readiness, substituters, archives, remote candidates, doctor results, platforms, trust, network state, executors, and parallelism.

The core owns:

- cache, substitution, archive, source-bundle, remote, local-build, and preflight route selection;
- ordered typed route rejections and blockers;
- requested, observed, policy, fallback, and executor concurrency limits;
- deterministic BLAKE3-bound effect plans;
- freshness and exact-effect observation checks.

Keep store probes, filesystem checks, doctor execution, key loading, remote command construction, host parallelism observation, mutations, execution, and report rendering in shell adapters.

Project remote candidate identity from `crunch-remote-core::RemoteCommand`. Do not open a remote session or redeem credentials during planning.

## Consequences

Equivalent facts produce the same route, job limit, rejected-reason order, effect identities, and plan identity. The pipeline observes host parallelism only when the user did not supply a job count.

A selected route does not prove execution. The shell must recheck the fact identity before each planned effect. It must reject stale facts, wrong effect identities, and route substitution without silent re-planning.

Existing route reports, reason codes, CLI behavior, and the 24-test route suite remain compatible.

This decision does not prove effect success, remote-worker correctness, output admission, route optimality, reproducibility, or release eligibility.
