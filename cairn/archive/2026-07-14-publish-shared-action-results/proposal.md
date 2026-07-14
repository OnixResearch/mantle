## Why

Mantle has canonical `mantle-action-spec-v1` identities, content-addressed objects, action receipts, and pure reuse admission, but it does not yet provide a durable shared index from an action ref to previously admitted results. Content-addressed storage alone cannot tell a fresh client which output objects a content-addressed action produced, and the current CA derivation mapping is local restart state rather than a signed cross-machine reuse contract.

Without a separate action-result layer, remote CAS and binary-cache sharing can transfer known objects but cannot provide general action-level memoization across developers, CI jobs, and clean clients.

## What Changes

- Add an immutable `mantle-action-result-v1` record binding one action ref to produced object refs, PathInfo and receipt refs, execution-policy evidence, producer identity, signatures, and bounded non-claims.
- Add a provider-neutral action-result index that maps an action ref to a bounded set of immutable candidate records instead of overwriting conflicting results.
- Treat index lookup as advisory discovery; every candidate still passes existing reuse, signature, producer-policy, object-completeness, reference-scan, sandbox, and network-policy admission.
- Publish action results only after ordinary output admission and atomically expose complete records without making local CA mappings authoritative.
- Detect differing admitted result sets for the same action ref and fail strong reuse with explicit nondeterminism evidence rather than selecting the last writer.
- Add typed Nickel policy for result sources, trust roots, bounds, offline behavior, and publication policy.
- Keep the storage/execution interfaces transport-neutral so a future adapter may map them to another protocol, but make no REv2 compatibility claim in this change.

## Impact

- **Surfaces**: build-correctness records and reports, `crunch-store` persistence, cache/substitution planning, build planning, typed Nickel policy, and local/HTTP cache sidecars.
- **Dependencies**: consumes accepted action-spec, action-receipt, CAS object, reuse-admission, PathInfo, and output-trust primitives.
- **Non-claims**: no proof that a cached producer was correct, no trust from index presence, no exactly-once publication, no REv2 compatibility, and no acceptance of mutable state as an undeclared action input.
- **Validation**: canonicalization/property tests, poisoning/conflict/partial-publication negatives, fresh-client shared-hit integration, offline behavior, trust-policy tests, and Cairn gates.
