# Store authority capabilities

Mantle splits store authority into concrete Rust values. Each value has private fields and named operations.

## Ownership table

| Capability | Owner | Allowed operations | Excluded operations |
|---|---|---|---|
| `BuildStore` | `crunch-build::Builder` | Resolve closures, transform castore nodes, calculate NAR facts, check caches, substitute outputs, and admit signed outputs. | Garbage collection, repair, source admission, arbitrary root mutation, backend replacement, and raw service access. |
| `ActionResultPort` | `crunch-build::Builder` | Discover, probe, admit, and publish action results through fixed backends. | Backend replacement, publisher replacement, and raw service access. |
| `BuildServiceStore` | Pipeline build-service wiring | Construct approved sandbox services and ingest fetched host paths. | Raw service access and store administration. |
| `OutputLookup` | `crunch-pipeline` | Find one exact `PathInfo` value and reject conflicting identities. | Output mutation and raw `PathInfoService` access. |
| `RootRegistry` | `crunch-pipeline` | Register a selected root only when its output exists. List retained roots. | Build execution, arbitrary store mutation, and raw service access. |
| `SourceStore` / `SourceAdmission` | CLI or source shell | Preflight and ingest verified sources. Adopt an approved local output. | Build execution and store administration. |
| `TransferStore` | Remote-transfer adapter | Ingest and render NARs, stream bounded blobs, and serialize admitted directory objects. | PathInfo administration, roots, GC, and publisher execution. |
| `AttestationStore` | Attestation command shell | Load artifact and runtime-closure attestations with layer checks. | Store mutation and raw services. |
| `ProvenanceStore` | Foreign-provenance shell | Run bounded castore provenance scans. | Build execution, administration, and publication. |
| `RustCacheStore` | Rust cache adapter | Ingest, verify, render, and export Rust-unit castore objects. | PathInfo, roots, substitution, and administration. |
| `PathInfoAdministration` | Verification and signing commands | Verify hashes and signatures or apply an explicit signing operation. | Blob and directory service access. |
| `StoreAdministration` / `StoreAdmin` | Operator shell | List, inspect, repair, transfer, archive, compose, retain, and garbage-collect through named operations. | Build realization and raw service escape. |
| `PublicationExecution` | Build application shell | Validate and execute a returned publication plan, then return typed observations. | Local output admission and plan construction. |

## Builder boundary

`Builder` owns only `BuildStore` and `ActionResultPort`. It does not own `StoreHandle` or any writable service trait object.

The pipeline keeps `OutputLookup` and `RootRegistry`. It uses these values after the builder finishes.

The pipeline creates sandbox services through `BuildServiceStore`. It does not receive blob, directory, or `PathInfo` services.

```text
application command
  -> role capability
  -> crunch-store shell
  -> private Snix services
  -> typed result or effect plan
  -> application shell
  -> checked effect execution
  -> typed observation
```

## Completed boundary

Production code outside `crunch-store` does not consume `StoreHandle`, `StoreHandleServices`, or raw Snix store-service traits. It receives concrete capability values or already-split capability sets.

Focused unit tests can use explicit fixture-only constructors. The architecture checker classifies these as fixtures and rejects the same access from runtime items.

`workspace_shell` now ingests snapshots through `BuildServiceStore`. Rust cache and delta-manifest adapters use Mantle-owned capability or in-memory fact boundaries instead of raw store services.

Local output admission returns a publication plan before any configured publisher executes. The shell records each publisher attempt as a typed observation.

## Migration rules

1. Select the capability that owns the required operation.
2. Add a named, bounded operation to `crunch-store` when no operation exists.
3. Do not return a writable service object or a broad callback.
4. Keep policy decisions in pure cores. Keep capability methods as thin I/O shells.
5. Keep `ActionResultPort` backends fixed after construction.
6. Keep output lookup and selected root registration outside `Builder`.

Source-policy and deterministic AST tests reject raw service escape, broad-handle runtime ownership, administrative build methods, and action-result backend replacement.

Compile-fail examples also reject garbage collection, repair, source import, root mutation, and raw blob access from `BuildStore`. Positive and negative publication tests prove plan-before-effect ordering and fail-closed effect identity.

## Compatibility and claim boundary

This split does not change store formats, report schemas, signatures, output identities, or supported build behavior.

The split proves only local Rust API reachability for the checked callers. It does not prove sandbox isolation or filesystem confinement.

It also does not prove output correctness, cache trust, reproducibility, provenance, availability, or release eligibility.
