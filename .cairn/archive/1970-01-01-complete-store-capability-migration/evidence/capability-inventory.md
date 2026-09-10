# Capability inventory — I1

Survey of every external `StoreHandle` consumer, raw Snix service escape, and
writable store authority as of `ce992851` (2026-09-09). Survey method:
`grep -rln StoreHandle` over `src/` and first-party `crates/`, then per-file
counts of raw-service accessor calls (`blob_service()`, `directory_service()`,
`pathinfo_service()`, `remote_pathinfo()`), then per-caller method-use
sampling.

## Already capability-only (no action)

| Caller | Capability | Operations used |
|---|---|---|
| `crates/crunch-build` builder path (non-orchestrate) | `BuildStore`, `BuilderStoreParts` | root registration, substitution reports, CA mapping, output node bookkeeping |
| `crates/crunch-pipeline` builder wiring | `PipelineStoreParts` | build-service construction, output persistence |
| `crates/crunch-build/src/orchestrate.rs` (most call sites) | `BuildStore` methods | `persist_and_export_signed_output`, attestation persistence, source admission |

## Escapes that must migrate

### Slice 1 — remote build (`src/remote_build.rs`)

- Owner intent: remote-build orchestration.
- Escapes: `store.directory_service()` ×7, `store.blob_service()` ×7,
  `store.pathinfo_service()` ×1. These feed closure materialization and NAR
  streaming for remote build requests.
- Allowed operation set (target): closure-node reads, NAR/stream reads for
  declared outputs, output-substitution reporting, build-service construction
  (`BuildServiceStore::bubblewrap_build_service` already exists).
- Migration order: 1 (largest raw-service consumer in the application shell).

### Slice 2 — remote transfer (`src/remote_transfer.rs`)

- Escapes: 4 raw-service accessor calls; 7 `StoreHandle` references.
- Allowed operation set (target): transfer-object reads/writes (chunk/blob
  ingestion, NDJSON frames), signed-PathInfo acceptance, attestation lookup.
- Migration order: 2.

### Slice 3 — store administration CLI (`src/store_cmd.rs`, `src/main.rs`)

- Escapes: `store_cmd.rs` 2 raw-service calls (sign/verify and GC paths),
  18 `StoreHandle` references; `main.rs` constructs `StoreHandleServices`
  (27 raw hits, mostly composition-root wiring).
- Allowed operation set (target): `StoreAdmin` (exists: root listing,
  migration), plus new administration operations for sign, verify, GC, pull,
  push. `main.rs` keeps construction but must pass capabilities, not the
  handle, into command modules.
- Migration order: 3.

### Slice 4 — pipeline and cache/archive adapters

- `crates/crunch-pipeline/src/lib.rs`: 1 raw-service call, 8 handle refs.
- `crates/crunch-rust-cache/src/lib.rs`: 1 handle ref, 12 raw-service-adjacent
  hits (PathInfo/NAR rendering for cache entries).
- `crates/crunch-build/src/orchestrate.rs`: 1 raw-service call remains after
  the capability slices landed there.
- Migration order: 4.

### Slice 5 — remaining application shells

`src/attest_cmd.rs` (4 raw hits), `src/bootstrap.rs` (13), `src/foreign_*`
(12 combined), `src/build_plan.rs`, `src/full_source_provider.rs`,
`src/source_built_fixed_point_shell.rs`, `src/foreign_provenance_audit.rs`.
Each keeps 1–7 handle references; most touch attestation lookup, output
lookup, and archive reads, which map to existing or planned capabilities
(`OutputLookup`, attestation lookup port, archive access).
- Migration order: 5.

## Writable store authority

Writable authority is reachable today by any code holding `StoreHandle`
(mutable methods: `insert_output_node`, `insert_built_output`,
`insert_ca_mapping`, persistence and signing). After migration, write access
must exist only inside `crunch-store` behind output-admission and
transfer-write capabilities. The compile-fail fixtures (I5) must reject any
application-port type that exposes these.

## Publication boundary (I4)

Output admission currently invokes configured `Publisher`s directly inside
persistence. Target: admission returns an admitted-output result plus an
ordered bounded publication effect plan; the application shell executes the
plan and records typed observations. Publisher failure must surface as a
failed observation without erasing admission.

## Non-claims

This inventory records source-level call shapes only. It does not prove
behavioral correctness of any path, and it is not evidence of publication or
release eligibility.
