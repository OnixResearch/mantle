# Materialize consumer-hosted effect components

## Why

Application-import components require a real consumer-owned host. Mantle must
materialize exact reviewed bytes without claiming that standalone Wasmtime
executed an unavailable host or broadening immutable import-free profiles.

## What Changes

- Extend the closed component toolchain with `molten-component-effects-v1`.
- Add explicit identity-bound consumer-host-required runtime validation.
- Preserve all source/lock/static/artifact/Structural+Bytes verification gates.
- Prepare a genuine dependency-free Rust/WIT effect fixture from pinned tools,
  with deterministic explicit input modes and reclaiming bounded guest memory.
- Keep canonical Preserves, runtime effect authority, and operated execution in
  the real Molten actor/system-extension consumers.

## Impact

Mantle component-pipeline/toolchain maintainers own the producer source, typed
bundle contract, deterministic fixtures, cohort, and immutable handoff. Octet
owns structural policy. Molten owns WIT/product semantics and host admission.
The immediate consumers are the real Molten actor and system-extension hosts;
their runtime evidence remains separate from producer materialization evidence.

## Non-Goals

No CLI mock host, synthetic bundle, baseline profile widening, unpinned checker
dependency, or execution claim from static validation is admitted. Publication
and native archival record only the observed bounded gates, not downstream
execution, runtime authority, release eligibility, or arbitrary guest behavior.

## Acceptance

The real bounded pipeline must produce and completely remeasure an effect-world
bundle after an actual immutable checker publication and coherent pin update.
The bundle exposes an exact typed unexecuted consumer-host requirement. Existing
standalone/import-free behavior and default serialization remain intact.
