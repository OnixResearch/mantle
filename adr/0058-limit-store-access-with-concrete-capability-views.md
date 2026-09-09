# ADR 0058: Limit store access with concrete capability views

## Status

Accepted

## Context

Mantle's `StoreHandle` combines castore and PathInfo services, remote substitution, action-result stores, publishers, source ingest, root retention, garbage collection, and mutable build-session state. `crunch-build::Builder` owns that broad handle, and pipeline code can retrieve it through `Builder::store_handle()`.

The [Atom Reforged architecture](https://nrd.sh/blog/atom-reforged.html) describes a read-only dependency firewall. Mantle cannot use a read-only store for all build work because realization must ingest and persist verified outputs. The useful principle is narrower: each subsystem receives only the store effects it owns.

## Decision Drivers

- Make authority visible in Rust types.
- Prevent future administrative methods from becoming available to build code by default.
- Keep raw writable service traits inside the store shell.
- Preserve current store formats, identities, and supported behavior.
- Avoid a new generic capability framework.

## Decision

Mantle will split store access into concrete capability values with private fields.

The initial roles are `BuildStore`, `OutputLookup`, `RootRegistry`, `SourceAdmission`, `ActionResultPort`, and `StoreAdmin`. `Builder` can own `BuildStore` and a fixed `ActionResultPort`. It cannot own `StoreAdmin`, `SourceAdmission`, or a broad compatibility handle.

Capability methods expose named high-level operations. They do not return writable blob, directory, PathInfo, publisher, or action-result backend objects.

Build-session output nodes, built-output facts, substitution reports, and CA resolution move into builder-owned session state or `BuildStore`. Pipeline orchestration retains output lookup and root registration separately.

A shell-owned compatibility facade can exist during migration. `Builder::store_handle()` will not remain part of the accepted design.

## Alternatives Considered

### Keep `StoreHandle` and rely on review

Rejected because every new public method silently expands builder authority.

### Add marker traits over `StoreHandle`

Rejected because callers can still regain broad access if the wrapped handle or raw service traits remain reachable.

### Make the complete build store read-only

Rejected because build realization must ingest transformed objects and persist verified outputs.

### Add a generic capability or effect framework

Rejected because concrete local roles are easier to review and do not add a repository-wide abstraction.

## Consequences

- Store operations move behind role-specific methods.
- Some tests need restricted test constructors instead of raw services.
- Pipeline and builder construction change, while wire formats stay stable.
- Compile-fail and source-policy tests become part of the authority boundary.
- The type split proves API reachability only. It does not prove filesystem or sandbox confinement.

## Update 2026-09-09: completed migration boundary

The store capability migration (change `complete-store-capability-migration`)
closed the remaining application-shell escapes and defined the final owners:

- `TransferObjectStore` is the transfer-object authority: NAR rendering,
  bounded blob reads, digest-verified canonical directory bytes, host-path
  and NAR ingestion, and output export. Remote transfer and remote build
  shells no longer call raw blob/directory/PathInfo accessors.
- Administration listing is a bounded shell operation
  (`store_list_pathinfos_bounded`); raw PathInfo services stay private.
- Output admission records a bounded, ordered `PublicationEffectPlan`
  instead of running publishers inline. The build orchestrator, as the
  application shell, drains and executes the plan and receives typed
  `PublicationObservation`s. A failed observation never changes the admitted
  output result.
- `tools/check_store_capability_boundary.rs` is the deterministic guard:
  zero raw-service escapes, zero writable-authority uses, and zero handle
  constructions outside the declared owners (CLI command shells, remote and
  foreign shells, pipeline orchestration, rust-cache adapter; builder
  orchestrator owns CA-mapping writes). `crunch-rust-cache` is recorded as a
  store-backed adapter that owns a private store instance.
- Migration removal: external `StoreHandle` broad consumers were reduced to
  declared owners; no external module reaches Snix blob, directory, or
  PathInfo services in production code.

Evidence scope and non-claims are unchanged: capability reachability and
observed effects only, not filesystem confinement, publisher honesty, or
release eligibility.
