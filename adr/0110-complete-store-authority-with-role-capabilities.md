# ADR 0110: Complete store authority with role capabilities

## Status

Accepted

## Context

ADR 0058 removed broad store authority from the builder and pipeline. Root command, remote transfer, attestation, bootstrap, provenance, Rust cache, archive, and compatibility shells still named `StoreHandle` or raw Snix services.

Configured output publishers also ran inside local persistence. Callers could not inspect an effect plan before publication, and publisher failures existed only as warning text.

## Decision

Mantle completes the migration with concrete role capabilities owned by `crunch-store`.

- `TransferStore` owns bounded NAR, blob, and directory transfer operations.
- `AttestationStore` owns read-only artifact and closure attestation lookup.
- `ProvenanceStore` owns bounded castore provenance scans.
- `RustCacheStore` owns Rust-unit cache castore ingest, completeness, render, and export operations.
- `SourceStore` and `ForeignRealizationStore` own source admission and foreign realization handoff.
- `PathInfoAdministration` and `StoreAdministration` own operator verification, signing, repair, transfer, archive, composition, root, and GC operations.
- Pipeline and planning constructors return already-split capability sets.

Raw Snix blob, directory, PathInfo, overlay, and database services remain inside `crunch-store`. Runtime modules outside that crate cannot name `StoreHandle` or those service traits. A deterministic Rust AST checker enforces this rule while classifying explicit test-support modules as fixtures.

Local output persistence returns `AdmittedOutput`, which contains truthful local `PathInfo` and a BLAKE3-bound `PublicationEffectPlan`. `PublicationExecution` validates the exact path, publisher count, order, index, and effect identity before it invokes publishers. Every attempt returns a typed `PublicationObservation`. A failed publisher does not erase local admission.

Public build-report JSON remains unchanged. Pipeline results retain typed observations in memory for application orchestration and tests.

## Consequences

- Production external broad-handle and raw-service findings become zero.
- Store-specific vendor translation stays at the store adapter boundary.
- Additional capability methods are narrow and reviewable rather than a generic service locator.
- Existing store bytes, signatures, paths, reports, command behavior, and effect order remain compatible.
- The type and source rails prove API reachability only. They do not prove output correctness, publisher honesty, filesystem confinement, remote trust, or release eligibility.
