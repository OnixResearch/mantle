# Change: Separate the remote-build hexagon

## Why

Mantle remote execution has strong trust and retry policy, but its application boundary remains mixed. `src/remote_build.rs` combines protocol records, state transitions, filesystem access, process execution, clocks, random identifiers, transport framing, credential handling, store access, output admission, and presentation.

The provider-neutral distributed seam also exposes Snix and `crunch-store` types. This makes infrastructure types part of application ports and prevents the compiler from enforcing an inward dependency direction.

## What Changes

- Add a dedicated `no_std + alloc` remote-build core for protocol admission, state transitions, retry, fencing, resource decisions, transfer decisions, output-admission inputs, effect planning, and receipt preimages.
- Add Mantle-owned application requests, observations, outcomes, blockers, effect plans, and port errors.
- Keep transport, filesystem, process, clock, random, credential, store, Snix, Tokio, and rendering types outside the core.
- Split remote application orchestration from stdio, SSH, local-executor, external-batch, store, persistence, and telemetry adapters.
- Translate Snix `PathInfo`, build requests, build results, and substitution reports only at adapter boundaries.
- Preserve existing protocol bytes and accepted behavior through compatibility projections and golden fixtures.
- Add positive, negative, replay, compile-fail, dependency, fault, and compatibility tests.

## Non-Goals

- Changing remote protocol versions or accepted wire bytes.
- Changing ticket, signer, output-trust, transfer, retry, resource, or scheduling policy.
- Replacing current remote transports or external batch providers.
- Claiming remote worker honesty, compiler correctness, successful effects, or output trust from a core decision alone.
- Duplicating active gateway, resource-policy, nominal-type, or Trellis verification work.

## Dependencies

- `complete-store-capability-migration` supplies narrow store and output-admission capabilities.
- Active remote-build changes remain owners of their behavior and evidence contracts.
- The accepted `remote-builds`, `build-scheduling`, and `realization-routing` requirements remain authoritative.

## Impact

- **Affected spec:** `remote-builds`
- **Affected code:** `src/remote_build.rs`, `crates/crunch-build/src/distributed*`, remote transfer, coordinator state, executor adapters, and remote tests
- **Compatibility:** accepted protocol bytes, identities, receipts, diagnostics, and remote behavior remain unchanged
- **Testing:** core transition tests, adapter contract tests, malformed protocol tests, replay tests, fault injection, architecture guards, and Cairn gates
