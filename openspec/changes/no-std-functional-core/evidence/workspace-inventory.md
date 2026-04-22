# Workspace Inventory: no-std functional core

Date: 2026-04-22

Scope: first-party crunch crates plus the root `crunch` package. Vendored `vendor/*`
workspace members stay out of the first-wave bucket decision because this
change only owns crunch-maintained crate boundaries.

## Bucket Summary

| Package | Bucket | Why this bucket now |
|---|---|---|
| `crunch-attestation-core` | `no_std now` | New first-wave core crate for attestation schema, canonicalization, digesting, and pure policy/validation transforms. |
| `crunch-project-core` | `no_std now` | New first-wave core crate for manifest, lock, merge, drift, upgrade, generated-input planning, and normalized refresh planning/outcome logic. |
| `crunch-attestation` | `split now` | Keeps discovery, file reads, witness-directory scanning, and std-facing compatibility surface outside the new core crate. |
| `crunch-project` | `split now` | Keeps resolver traits/implementations, file reads/writes, tempdirs, and shell-facing project APIs outside the new core crate. |
| `crunch` | `shell only` | CLI parsing, tracing, filesystem paths, process orchestration, and report formatting are all std/host concerns. |
| `crunch-eval` | `shell only` | Nickel runtime, file loading, import-path handling, and backend orchestration depend on host/runtime facilities; not first-wave pure-core scope. |
| `crunch-glue` | `shell only` | Current job is converting evaluated derivations into `nix_compat` store-path data; keep this boundary stable while attestation/project pattern lands first. |
| `crunch-build` | `shell only` | Bubblewrap/process execution, fetchers, sandbox inputs, and build orchestration are explicitly host-effectful. |
| `crunch-store` | `shell only` | Filesystem export, redb state, cache/substitution HTTP, GC roots, and castore persistence are std/runtime owned. |
| `crunch-pipeline` | `shell only` | Async eval/build/store orchestration, mutation locks, channels, and root forcing policy stay in the imperative shell tier. |
| `crunch-delta` | `shell only` | Protocol work is interesting later, but current crate still mixes protocol models with retained-content stores, substitution fixtures, and runtime-facing integration helpers. |
| `crunch-shell` | `shell only` | Has pure planning value, but current public API is built around `PathBuf`, `OsString`, `std::env::split_paths`, and shell-activation host semantics. |

## Std-Owned Reasons For Non-Core Buckets

### `split now`

- `crunch-attestation`
  - Owns `discovery.rs` path traversal, `std::fs` reads, and std error/reporting.
  - Must preserve the current std-facing compatibility layer while core types and transforms move underneath it.
  - Also best place for a future std-only extension trait that preserves today's `Canonicalize` ergonomics if the core crate drops public traits.

- `crunch-project`
  - Owns refresh resolver traits and all git/URL/local-file hashing I/O.
  - Owns lockfile writes, generated-input file writes, tempdirs, and CLI-facing adapters.
  - Also owns project-attestation synthesis today because that layer depends on the std-facing attestation crate surface.

### `shell only`

- `crunch`
  - `clap`, tracing, cwd/path handling, JSON/text output, and command dispatch are intentionally std-only shell work.

- `crunch-eval`
  - Nickel evaluation still depends on runtime file access, import-path discovery, backend selection, and error rendering around the `nickel-lang` runtime.

- `crunch-glue`
  - Conversion currently sits on store-path formatting and `nix_compat` interop rather than a self-contained pure-core boundary worth splitting before attestation/project.

- `crunch-build`
  - Sandbox/process execution, fetchers, network access, and derivation build orchestration are effectful by definition.

- `crunch-store`
  - Local store mutation, redb state, castore export, substitution, and attestation persistence all depend on filesystem/network/runtime services.

- `crunch-pipeline`
  - Async orchestration, mutation locks, worker scheduling, and root-build lifecycle control are imperative-shell responsibilities.

- `crunch-delta`
  - First-wave no-std change is about proving the pattern on existing pure domains, not widening scope into protocol/runtime integration at the same time.

- `crunch-shell`
  - Current boundary types (`PathBuf`, `OsString`) and PATH/env composition rules are explicitly host-shaped; later extraction would need a separate API-shape redesign.

## API Ownership After Current Extraction Step

### `crunch-attestation-core` now owns

- `Error`
- `AttestationDigest`
- `SchemaVersion`
- attestation schema data types:
  - `Claims`
  - `Node`, `NodeKind`
  - `Edge`, `EdgeKind`
  - `ArtifactFacts`, `ArtifactAttestation`, `ArtifactReference`
  - `ClosureSemantics`, `ClosureFacts`, `ClosureAttestation`
  - `ProjectFacts`, `ProjectAttestation`

### `crunch-attestation` still owns

- `discovery.rs` filesystem scanning and file loading
- `canonical.rs` canonicalization logic and current `Canonicalize` trait surface
- `release.rs` release/witness attestation transforms
- `policy.rs` policy/revocation evaluation and witness filtering
- std-facing re-export/adaptor modules in `src/{error,digest,schema,version}.rs`

### `crunch-project-core` now owns

- `SchemaVersion` and `parse_version(...)`
- manifest data model and validation:
  - `ProjectManifest`
  - `ManifestInput`
  - `InputKind`
  - `GitReference`
  - `HashAlgo`
  - `HashSpec`
  - `PatchDef`
  - `PatchSource`
- lockfile data model and validation:
  - `Lockfile`
  - `LockEntry`
  - `LockedKind`
  - `LockedHash`
  - `LockedPatch`
  - `LockedPatchSource`

### `crunch-project` still owns

- `refresh.rs` resolver traits, git/url/local-file hashing boundary, and refresh outcome application
- `attestation.rs` project-attestation synthesis
- `generate.rs`, `merge.rs`, `drift.rs`, `upgrade.rs`, and `mirrors.rs` until their APIs are reshaped for core rules
- std-facing re-export/adaptor modules in `src/{manifest,lock,version}.rs`

## Immediate Extraction Notes

- `crunch-attestation` current `Canonicalize` public trait will not survive unchanged inside a first-wave core crate because the OpenSpec API-shape rules forbid public traits in core crates.
- `crunch-project` current refresh boundary cannot move as-is because `RefreshResolver` is a public trait and current core-facing functions use trait objects and references; the std adapter must keep resolver traits while the core takes normalized data.
- `crunch-project` and `crunch-attestation` both have public inherent methods with reference receivers today (`&self`), so the later extraction needs an explicit API-shape pass, not a file copy.
