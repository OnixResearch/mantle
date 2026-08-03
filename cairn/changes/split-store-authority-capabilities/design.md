# Design: Split store authority into narrow capabilities

## Context

`StoreHandle` combines durable services, remote cache configuration, action-result stores, publishers, garbage-collection roots, and mutable build-session maps. `Builder` owns this broad value and returns it through `store_handle()`. The pipeline then uses the raw PathInfo service and root methods after a build.

The goal is authority separation, not a rename. A wrapper that still returns `Arc<dyn BlobService>`, `Arc<dyn DirectoryService>`, or `Arc<dyn PathInfoService>` would preserve the same write authority.

## Decisions

### Decision: Use concrete capability values

**Choice:** `crunch-store` will expose concrete values with private fields for these roles:

- `BuildStore`: high-level input resolution, castore transformation, cache lookup, substitution, CA mapping, and signed output admission;
- `OutputLookup`: read-only PathInfo and output queries needed by reports;
- `RootRegistry`: bounded root registration and retention queries;
- `SourceAdmission`: verified source preflight and import;
- `ActionResultPort`: discovery, probing, admission, and publication through fixed backends;
- `StoreAdmin`: garbage collection, repair, pin management, migration, and backend configuration.

`Builder` may own `BuildStore` and `ActionResultPort`. It will not own `StoreAdmin`, `SourceAdmission`, or a raw service value.

**Rationale:** Concrete values make the allowed operations visible in Rust method lookup. They avoid a new generic framework and keep dynamic service dispatch private to `crunch-store`.

### Decision: Keep raw services private

**Choice:** Capability methods will perform named high-level operations. Build and pipeline crates will not receive writable service trait objects.

Castore rewrites, NAR calculation, path ingestion, closure resolution, and output persistence will move behind methods owned by the matching capability. A temporary compatibility facade can remain in CLI-owned code during migration, but `Builder::store_handle()` will be removed.

**Rationale:** Returning a service trait object defeats the authority split because the service trait includes mutation.

### Decision: Separate durable services from build-session state

**Choice:** Build-session maps for output nodes, built outputs, substitution reports, and CA resolution will live in a builder-owned session value or inside `BuildStore`. Durable store services and administrative state will remain shared through private internals.

**Rationale:** Session state belongs to one realization run. Keeping it out of the administrative handle reduces hidden cross-run mutation.

### Decision: Keep root registration outside build execution

**Choice:** Pipeline orchestration will retain a `RootRegistry` capability and an `OutputLookup` capability before it starts a builder. After execution, it can register selected outputs through one high-level `register_if_present` operation.

**Rationale:** A build can produce an output without receiving authority to pin arbitrary existing store paths.

### Decision: Prove the API boundary and runtime compatibility separately

**Choice:** Positive tests will exercise local builds, cache hits, substitution, CA outputs, action-result reuse, publication, and selected root registration. Compile-fail examples and source-policy tests will reject GC, source import, backend replacement, raw service access, and arbitrary root mutation from build code.

Golden or equivalent comparisons will preserve report fields, PathInfo facts, output paths, and canonical identities.

**Rationale:** A restricted API is not enough if behavior changes. Runtime success is not enough if broad authority remains reachable.

## Functional core and imperative shell

Pure decision functions continue to own cache, admission, transition, and identity decisions over supplied facts. Capability values are thin imperative shells that perform approved I/O and apply pure plans. They must not move policy decisions into service adapters.

## Risks and trade-offs

- Moving service-dependent helpers can create a large diff. Migrate one operation family at a time.
- Shared internals can hide authority if capability methods become generic escape hatches. Source guards must reject broad `with_service` or raw-service methods.
- Pipeline callers can lose test setup flexibility. Test-only constructors must still return the same restricted capabilities.
- Active action-result changes can conflict with this work. Rebase after those call paths stabilize.

## Claim boundary

This change proves only that selected Rust callers cannot name operations absent from their capability type. It does not prove sandbox isolation, filesystem confinement, process authority, output correctness, cache trust, or release eligibility.
