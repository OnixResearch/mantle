# ADR 0024: Separate shared action results from CAS and execution

## Status

Accepted

## Context

Mantle has canonical action specs and receipts, content-addressed object storage, local CA derivation mappings, binary-cache substitution, and local/remote executors. These surfaces answer different questions:

- CAS answers whether identified content exists;
- an action-result index answers which admitted result records claim to satisfy an action identity;
- an executor performs an action when no candidate is admitted.

Content-addressed outputs do not by themselves let a fresh client discover which outputs an action produced. Conversely, an index hit cannot prove that referenced content exists or is trustworthy. Treating a local CA mapping or remote index operator as authority would collapse discovery, content, and trust into one mutable surface.

## Decision Drivers

- Enable shared action-level memoization for clean clients and CI jobs.
- Preserve BLAKE3 action/object identities and existing PathInfo, receipt, attestation, and reuse-admission semantics.
- Expose nondeterministic or conflicting action results instead of hiding them with last-writer-wins updates.
- Keep local, HTTP, native remote, and possible future protocol adapters outside pure identity and admission logic.
- Preserve Mantle's frontend-neutral build-tool boundary.

## Decision

Mantle will keep three explicit backend-neutral interfaces:

1. **Object store** — immutable content lookup and transfer by canonical object ref.
2. **Action-result store** — bounded lookup and publication of immutable signed result-record refs by canonical action ref.
3. **Executor** — local or remote realization of a declared action after planning and reuse admission miss.

Action-result lookup is advisory. Every candidate must pass ordinary object completeness, PathInfo, receipt linkage, signature, producer-policy, sandbox/network-policy, reference-scan, and claim-strength admission before Mantle skips execution.

An action ref may index multiple immutable candidates. Identical candidates deduplicate by result ref. Differing otherwise admissible output sets are preserved and cause strong reuse to fail with explicit nondeterminism evidence; source order and last writer do not select a winner.

Publication occurs only after output admission and makes complete records visible atomically. Existing local CA mappings remain restart hints and may be promoted only by reconstructing and admitting a complete result record.

Mantle-native local and HTTP sidecars are the initial shells. REv2 or another protocol may be added later only through a separate adapter change with compatibility evidence; this ADR makes no protocol-compatibility claim.

## Implementation

- `crates/crunch-action-result-core` owns bounded records, BLAKE3 identities,
  candidate validation, deduplication, conflict classification, and strong
  reuse planning without CAS, executor, or transport dependencies.
- `crates/crunch-store/src/action_result.rs` owns the distinct
  `ActionResultStore` interface and atomic local/bounded HTTP shells. It does
  not implement `PathInfoService` or `BuildService`.
- `crates/crunch-build/src/orchestrate.rs` joins the interfaces at the
  pre-execution admission boundary and publishes only after output admission.
- `crates/crunch-action-result-core/tests/architecture.rs` guards the pure-core
  dependency boundary. Clean-client and zero-executor-call tests guard the
  runtime separation.

## Alternatives Considered

### Use CAS object presence as the action cache

Rejected because a content ref does not identify which action produced it, while a content-addressed action output may be unknown to a clean client before execution.

### Publish only one mutable result per action

Rejected because overwrite order would hide nondeterminism, producer disagreement, and cache poisoning.

### Make the coordinator or cache server an output-trust root

Rejected because transport/resource authority is separate from producer and output admission.

### Adopt REv2 as Mantle's semantic core

Rejected because Mantle's BLAKE3 identity, PathInfo, attestation, store-prefix, and claim boundaries must remain authoritative. A future adapter can translate bounded compatible surfaces without redefining core semantics.

## Consequences

- Clean clients can discover shared action results without prior local CA mappings.
- Metadata and GC policy become more complex because result records and indexes are separate from output objects.
- Conflicting action results become visible failures for strong reuse rather than silent cache selection.
- Executors remain replaceable behind the build-service boundary, while shared reuse semantics stay consistent across local and remote routes.
