# Change: Add evidence-driven remote resource policy

## Why

Static resource requests are necessary admission inputs, but they do not use Mantle's prior observations to select an efficient machine class. Hosted build services show the value of historical selection, bounded OOM recovery, usage accounting, and authorized result reuse.

These mechanisms are safe only when decisions are deterministic, versioned, replayable, and linked to explicit observations. Mantle must not turn opaque heuristics, arbitrary retries, or shared storage into unreviewed execution authority.

## What Changes

- Record bounded resource observations for completed remote attempts.
- Add a pure, versioned selection policy that combines declared requirements, eligible machine classes, quota facts, and compatible historical observations.
- Add bounded escalation after positive OOM evidence only.
- Reserve and reconcile project and account usage with explicit units and reason codes.
- Add explicit cross-project result-sharing scopes while preserving signature, policy, platform, identity, and CAS checks.
- Publish resource decisions, retries, accounting, and reuse through the Valence build-service evidence profile.
- Add a license-reviewed benchmark corpus inspired by `nixbench` operating patterns without copying opaque service policy or incompatible assets.
- Gate compatibility, performance, security, and fault evidence through Cairn.

## Non-Goals

- Machine learning or an opaque selection service.
- Retry after arbitrary build failure.
- Unlimited escalation or unpublished limits.
- Treating peak memory as a proof of future sufficiency.
- Cross-tenant reuse based only on a matching output path or shared CAS presence.
- Claims that benchmark success proves production performance or correctness.

## Dependencies

- `add-nix-remote-service-gateway` for request and observation APIs.
- The accepted Valence build-service evidence profile.
- OnixOS machine-class declarations.
- ChaosControl campaigns for scheduler, OOM, worker-loss, and storage fault behavior.

## Impact

- **Affected specs:** `remote-builds`
- **Affected code:** `crunch-resource-policy-core`, `crunch-resource-policy`, coordinator and fenced-attempt adapters, action-result integration, machine contracts, Nickel policy, Valence projection, benchmarks, fault fixtures, documentation, and lifecycle gates
