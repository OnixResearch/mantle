# ADR 0107: Bound pipeline growth before root effects

- **Status:** Accepted
- **Date:** 2026-09-01

## Context

`crunch-pipeline` collects streamed evaluation roots and converts completed
build outcomes into managed root registrations. The earlier implementation
validated final root counts but let the collection grow before that check. It
also built the registration batch inside the effectful root-registry function.

A Tiger Style repair must not change evaluation results, root classes,
substitution labels, generation metadata, cache-only trust, or effect order.
Assertions over untrusted input would add abort authority and are not an
acceptable substitute for admission.

## Decision

Pass the evaluation session's admitted root count into message collection.
Reserve that capacity and return a typed error before an extra root is added.
Keep the existing final completeness checks.

Derive managed-registration capacity with checked arithmetic over supplied
outputs and eligible source paths. Build and deduplicate the complete plan
before root-registry I/O. Assert only the internal plan facts that each inserted
path has one registration and that the plan fits its derived capacity.

Keep orchestration capabilities grouped in the existing builder bundle. Use
named records only for private managed-generation and failure-key helpers. Keep
public APIs unchanged.

Preserve this effect order:

1. complete worker execution;
2. prepare and commit the managed batch;
3. register selected retained outputs;
4. return build and evidence results.

## Consequences

- Eager collection has an explicit root bound before growth.
- Managed-root planning is deterministic and separate from persistence.
- Duplicate output or source paths keep first-observation behavior.
- Invalid capacity arithmetic returns `Error::Build` instead of wrapping.
- Cache-only strict policy and public pipeline interfaces do not change.
- This decision proves local admission and ordering only. It does not prove
  builder correctness, store durability, source trust, or release eligibility.
