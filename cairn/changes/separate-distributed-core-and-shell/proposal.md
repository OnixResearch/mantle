# Change: Separate distributed core and shell

## Why

`crunch-build::distributed` states that it is a pure provider-neutral module. The module also exposes Snix and store implementation types, async provider ports, concrete adapters, and effectful orchestration.

Examples include `VerifiedRealizationArtifact`, `ArtifactResolver`, `ArtifactPublisher`, `DerivationRealizer`, `RemoteBuildServiceAdapter`, and `resolve_before_dispatch`.

The external batch path has a related ownership problem. `ExternalBatchDispatcher` lives with direct-process and Slurm implementations, returns raw `String` failures, and is consumed by remote coordination code.

These structures mix deterministic build meaning with infrastructure types, async calls, adapter selection, and failure translation.

## What Changes

- Define provider-neutral distributed build facts, decisions, plans, and typed domain errors.
- Keep Snix, store, process, Slurm, async runtime, and protocol implementation types in shells and adapters.
- Put genuine outbound contracts with the application code that consumes them.
- Make route, fallback, admission, and outcome decisions pure over explicit values.
- Keep adapter selection and call ordering in visible composition roots.
- Replace raw dispatcher failures with typed infrastructure and application outcomes.
- Add positive and negative tests at core, shell, and adapter boundaries.

## Non-Goals

- Replacing the Nix store protocol or changing accepted PathInfo meaning.
- Removing Snix or `crunch-store` from Mantle infrastructure.
- Changing canonical bytes, BLAKE3 identities, route policy, or external batch protocol versions.
- Claiming remote execution, cache integrity, scheduler correctness, or output trust from adapter success.

## Impact

- **Affected specs:** `realization-routing`, `external-batch-dispatchers`
- **Affected code:** distributed realization, resolver and publisher integration, remote build services, external batch coordination, and Slurm/direct-process adapters
- **Compatibility:** preserve supported public behavior through explicit projections and temporary compatibility adapters
- **Testing:** pure decisions, shell order, vendor translation, malformed data, unavailable services, timeout, cancellation, ambiguity, and no-fallback cases
