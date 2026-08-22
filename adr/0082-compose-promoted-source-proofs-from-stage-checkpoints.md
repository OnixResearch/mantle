# ADR 0082: Compose promoted source proofs from stage checkpoints

## Status

Accepted (2026-08-22)

## Context

The source-built fixed-point proof has six ordered stages. Earlier policy required every promoted attempt to execute all six stages in one fresh process.

Mantle already has receipt-validated dev provider reuse, persistent content-addressed stores, shared action-result admission, and an unfinished fresh-directory resume change. Those mechanisms deliberately cannot satisfy a promoted proof.

Repeated promoted attempts have reconstructed identical StageX, native-provider, and Rust-provider work after failures in later receipt or validation code. Raw store reuse would be too weak, but rebuilding admitted stage evidence is unnecessary.

## Decision Drivers

- Reuse completed provider work after unrelated later-stage source changes.
- Preserve exact source, recipe, policy, resource, predecessor-output, and semantic-output authority.
- Retain original protected-execution and build evidence.
- Never present restored work as current-attempt execution.
- Restore into a fresh proof root and remeasure every payload.
- Keep raw cache entries and dev state ineligible for promoted claims.

## Decision

Mantle publishes one promoted provider checkpoint after the first four stages complete. The checkpoint contains ordered stage records and the complete StageX, provider, Rust, admission, transcript, and closure payload.

Each stage record has a stage-specific authority digest. The provider lookup key contains only source and policy inputs used by those four stages. It also contains a provider-recipe projection over `bootstrap/`, `builders/`, and `lib/`.

Mantle source and vendor inputs do not enter the provider lookup key because they first affect stage1. A Mantle-only edit can reuse provider work. A provider recipe, StageX source, native source, Rust source, policy, or resource change cannot.

A lookup key discovers candidates but grants no authority. Mantle remeasures each candidate and admits only promoted origin with complete prior execution evidence and exact payload identity. Multiple candidates with equal semantic outputs are equivalent. Conflicting semantic outputs fail closed.

An adopting attempt copies the selected payload into a fresh root. It remeasures the restored bytes before stage1. The final receipt marks the first four stages as `restored-checkpoint` and binds the checkpoint plus original execution-evidence digests.

A preserved stopped attempt can seed the checkpoint store only after its status, plan, stage authorities, recipe projection, provider admissions, receipts, and payloads revalidate. A running attempt cannot be imported.

## Alternatives Considered

### Permit the dev provider cache in promoted runs

Rejected. Dev entries do not carry the complete promoted execution tree or final stage-evidence composition.

### Use content-addressed store presence as proof

Rejected. Content identity does not establish producer, action, policy, or execution evidence.

### Resume the failed staging directory in place

Rejected. Mutable attempt state does not provide fresh-root confinement or immutable checkpoint publication.

### Reuse every action through the shared action-result index

Deferred. This is valid for ordinary derivations, but it does not restore the complete StageX execution tree. The provider checkpoint gives the first useful bounded boundary.

## Consequences

- Later proof retries can begin at stage1 after checkpoint verification and restore.
- Checkpoint publication copies substantial evidence once, but avoids repeated provider construction.
- The proof claim becomes compositional across admitted stage executions.
- A restored stage was not executed in the adopting attempt.
- Cold execution remains available when no checkpoint store is selected.
- This decision does not prove compiler correctness, independent rebuild agreement, or release reproducibility.
