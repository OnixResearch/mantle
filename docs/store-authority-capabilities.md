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
| `SourceAdmission` | CLI or source shell | Preflight and ingest verified sources. Adopt an approved local output. | Build execution and store administration. |
| `StoreAdmin` | Operator shell | Run garbage collection with explicit retained castore roots. | Build realization. |

## Builder boundary

`Builder` owns only `BuildStore` and `ActionResultPort`. It does not own `StoreHandle` or any writable service trait object.

The pipeline keeps `OutputLookup` and `RootRegistry`. It uses these values after the builder finishes.

The pipeline creates sandbox services through `BuildServiceStore`. It does not receive blob, directory, or `PathInfo` services.

## Compatibility facade

`StoreHandle` remains a shell-owned compatibility facade. CLI and operator shells can split it into narrow capabilities.

Archive, repair, pin, migration, and transfer shells can use this facade during migration.

Focused unit tests can use test-only service constructors. These constructors use `#[cfg(test)]` and do not change the production API.

`workspace_shell` remains an explicit shell compatibility boundary. It can adapt services required by the existing workspace protocol.

## Migration rules

1. Select the capability that owns the required operation.
2. Add a named, bounded operation to `crunch-store` when no operation exists.
3. Do not return a writable service object or a broad callback.
4. Keep policy decisions in pure cores. Keep capability methods as thin I/O shells.
5. Keep `ActionResultPort` backends fixed after construction.
6. Keep output lookup and selected root registration outside `Builder`.

Source-policy tests reject raw service escape, broad builder ownership, administrative build methods, and action-result backend replacement.

Compile-fail examples also reject garbage collection, repair, source import, root mutation, and raw blob access from `BuildStore`.

## Store backend selection

The capability views do not change with the store backend.

Only the CLI composition root chooses `--store-backend`. `StoreConfig` requires the backend, and no library constructor supplies a default. See [Store backends](store-backends.md).

## Compatibility and claim boundary

This split does not change store formats, report schemas, signatures, output identities, or supported build behavior.

The split proves only local Rust API reachability for the checked callers. It does not prove sandbox isolation or filesystem confinement.

It also does not prove output correctness, cache trust, reproducibility, provenance, availability, or release eligibility.
