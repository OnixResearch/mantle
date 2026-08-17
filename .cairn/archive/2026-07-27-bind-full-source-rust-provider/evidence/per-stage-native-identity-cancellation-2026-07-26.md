# Per-stage native-identity cancellation

## Scope

This evidence records an operator cancellation before provider publication.
It does not prove provider completion.

## Reason

Detached v8 retained digest-bound stage plans, scripts, manifests, receipts, metadata, and smoke summaries.
Its stage construction identity still omitted explicit native-provider, source-closure, admission-report, and host-tool-manifest digests.

The accepted requirement applies these bindings to every Rust stage.
A final global binding cannot replace a missing per-stage commitment.

## Cancellation

Pueue task 457 terminated the detached v8 process group before publication.
The same command verified that no provider output existed.
The incomplete scratch had no validated final provider receipt.

## Repair

Every full-source stage construction identity must also record:

- native provider ID
- native provider metadata BLAKE3
- native provider output BLAKE3
- source-closure manifest BLAKE3
- admission-report BLAKE3
- host-tool manifest BLAKE3

Full-source binding validation must compare every value with the independently validated binding inputs.
Negative tests must reject native-provider and source-closure substitution.

## Next action

A fresh construction must create receipts with the repaired per-stage native bindings.
The v8 scratch cannot be resumed as completion evidence.
