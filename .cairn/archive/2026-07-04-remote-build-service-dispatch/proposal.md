## Why

Remote dispatch currently sits outside the lazy goal scheduler as a client-side stdio loop over evaluated roots. That prevents normal deduplication, per-derivation local/remote routing, dependency interleaving, and scheduler-level accounting.

## What Changes

- Extract remote execution into a scheduler-compatible remote build service.
- Route ready derivation goals through local sandbox, fetcher, substitution, or remote execution without changing goal identity.
- Import remote results through the same signed PathInfo, artifact-attestation, and store persistence path used by local/substituted outputs.
- Preserve concrete-build-only behavior: the remote service receives already evaluated derivation/action inputs, never raw Nickel or frontend module semantics.

## Impact

- **Files**: `crunch-build` dispatch service boundary, worker goal dispatch, remote-build client adapter, store import path, build reports, and Cairn remote-builds spec delta.
- **Testing**: positive scheduler dispatch and dedupe fixtures; negative raw-eval request, mismatched output, untrusted output key, remote failure classification, and local fallback fixtures.

## Out of Scope

- Long-running coordinator scheduling.
- Production P2P listener.
- Rewriting the lazy goal scheduler into an eager global DAG.
