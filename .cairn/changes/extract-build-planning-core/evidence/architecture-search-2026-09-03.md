# Build-planning architecture search

Source commit: `d3fa55f20da8deaa5d30f9e97ecf6028d4b83080`

## Question

Which existing component can own Mantle's realization-route and concurrency decisions without host authority?

## Inspected evidence

- `src/realization_routing.rs` already contains the accepted pure route order, remote and source candidate policy, upload limits, reason codes, and route reports.
- `src/build_plan.rs` gathers store, action-result, doctor, trust, source, and remote facts, but it still selected cache, substitution, local-build, and preflight actions in the shell.
- `crates/crunch-pipeline/src/lib.rs::resolve_max_jobs` called `std::thread::available_parallelism` inside the clamp policy.
- `crunch-composition-core` owns bounded castore composition, not route eligibility or concurrency.
- `crunch-remote-core` supplies Mantle-owned remote command and identity values. It does not own build-route preference.
- No sibling OnixResearch component owns Mantle's realization-route and job-limit semantics.

## Decision

Promote the existing route kernel intact into `crunch-build-planning-core`. Extend that `no_std + alloc` core with explicit observation values, typed blockers, deterministic job-limit policy, plan-bound effects, freshness checks, and observation acceptance.

Keep the root route module as a compatibility re-export. Move cache, substitution, build, and preflight action selection behind a core function. Keep host parallelism observation in the pipeline shell and send its checked value to the core.

Use a remote adapter that projects `crunch-remote-core::RemoteCommand` identity plus explicit capability and trust facts. It must not open a remote session or redeem a credential.

## Non-claims

The extraction does not prove that an effect ran, a remote worker succeeded, an output was admitted, a selected route was optimal, or a release is eligible.
