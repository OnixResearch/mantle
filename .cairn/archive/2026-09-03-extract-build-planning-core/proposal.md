# Change: Extract the build-planning core

## Why

Mantle has deterministic routing and scheduling kernels, but some application decisions still share modules with store probes, filesystem checks, doctor diagnostics, key loading, and shell errors. The pipeline also reads host parallelism inside `resolve_max_jobs`, which combines ambient observation with policy.

Build planning needs one explicit boundary from observed facts to route, concurrency, and effect decisions. The shell must gather facts, while the core must remain deterministic and host-capability-free.

## What Changes

- Move cache, substitution, source-bundle, remote, local-build, and preflight route selection into a pure application core over explicit observations.
- Make requested jobs, observed host parallelism, and policy limits explicit inputs to deterministic concurrency policy.
- Return typed selected routes, rejected-route reasons, job limits, blockers, and effect plans.
- Keep store probes, filesystem checks, doctor execution, key loading, environment reads, remote discovery, and report rendering in the shell or adapters.
- Prevent planning from opening sessions, redeeming credentials, mutating stores, executing builds, or publishing outputs.
- Preserve current route order, reason codes, job limits, reports, and CLI behavior through compatibility fixtures.
- Add positive, negative, deterministic replay, hidden-state, and architecture tests.

## Non-Goals

- Changing route preference, trust, network, source, cache, remote, or local-executor policy.
- Changing scheduler priority or remote resource admission.
- Treating a selected route or effect plan as evidence of execution success.
- Adding runtime provider discovery to the core.
- Creating one port for every local observation helper.

## Dependencies

- The accepted `realization-routing` and `build-scheduling` requirements define route and scheduling semantics.
- `complete-store-capability-migration` supplies bounded store observation capabilities.
- `separate-remote-build-hexagon` supplies application-owned remote candidate and output-trust facts.

## Impact

- **Affected specs:** `realization-routing` and `build-scheduling`
- **Affected code:** `src/build_plan.rs`, `src/realization_routing.rs`, `crates/crunch-pipeline`, doctor and store observation adapters, and build-plan tests
- **Compatibility:** accepted routes, job limits, reports, reason codes, and command behavior remain unchanged
- **Testing:** pure route tests, parallelism tests, hidden-state negatives, shell observation tests, compatibility fixtures, and Cairn gates
