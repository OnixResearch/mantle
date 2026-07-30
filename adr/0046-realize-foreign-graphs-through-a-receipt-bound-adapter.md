# ADR 0046: Realize foreign graphs through a receipt-bound adapter

## Status

Proposed

## Context

Mantle can import, validate, admit, and plan concrete foreign derivation graphs. It cannot yet execute those graphs with exact translated identities.

A direct store-prefix replacement is insufficient. Each parent must reference the target path that Mantle computes for its child. Source bytes, sandbox policy, scheduler effects, store admission, and output audit also have different evidence boundaries.

Guix and Nix frontend evaluation must remain outside Mantle consumption. OnixOS must continue to own system configuration and boot assembly under ADR 0010.

## Decision

Mantle will realize foreign derivation graphs through a thin, receipt-bound adapter.

A pure compiler will parse concrete prefix-aware ATerm data, order the graph, lower supported builtins, and map exact foreign derivation, output, and source paths. It will emit resolved native units before any build effects occur.

The realization shell will verify source records and per-derivation execution profiles. Each profile digest will contribute to target derivation identity. The shell will then register units in Mantle’s ordinary derivation registry and call the existing scheduler and store paths.

The initial Guix profile will not provide ambient `/bin/sh`. Compatibility exceptions will be typed, bounded, per derivation, and included in evidence.

A separate castore-backed provenance audit will classify executable payloads and reject unresolved foreign references. Audit success will not promote static path observations into package correctness or runtime behavior.

Import receipts, executable plans, realization receipts, build reports, store attestations, and provenance audits will remain separate artifacts. Each artifact will name its bounded claim and non-claims.

## Consequences

Mantle reuses its scheduler, build services, castore, PathInfo, and attestation machinery. It does not gain a Guix evaluator, Nix evaluator, package module layer, or second execution engine.

Execution-profile changes will change target derivation identities. This prevents one target path from hiding different sandbox semantics.

Foreign source inputs must be present as verified Mantle source records or fixed-output fetch units. Ambient `/gnu/store` and `/nix/store` content cannot satisfy source requirements during consumption.

The adapter can report admitted, planned, realized, and provenance-audited states without merging those claims. OnixOS remains responsible for initrd, activation, accounts, Shepherd, VM, deployment, and boot evidence.

## Alternatives Considered

### Emit Nickel or Nix expressions and use the normal evaluator

Rejected. This would add frontend translation semantics and require another evaluator during consumption.

### Add a separate foreign scheduler

Rejected. It would duplicate goal ordering, cancellation, substitution, store admission, and evidence logic.

### Replace only the foreign store prefix

Rejected. Prefix replacement can create parent references that do not match recomputed child paths.

### Keep execution profiles as unhashed side metadata

Rejected. The same target path could then represent different execution semantics.

### Provide `/bin/sh` globally for compatibility

Rejected. This would hide Guix compatibility gaps and weaken per-derivation policy.

### Treat successful realization as provenance verification

Rejected. Builders can emit untranslated references or unclassified executable content after graph compilation.
