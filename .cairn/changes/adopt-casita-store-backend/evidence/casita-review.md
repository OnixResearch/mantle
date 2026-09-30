# Evidence: Casita source review

## Sources

- Article: "Introducing Casita: A content-addressed store for source code and
  build artifacts", https://casita.rs/blog/introducing-casita-a-content-addressed-store-for-source-code-and-build-artifacts/
- Division of responsibility: https://casita.rs/concepts/responsibilities/
- Garbage collection: https://casita.rs/concepts/garbage-collection/
- Rust library guide: https://casita.rs/library/
- Rust API reference: https://casita.rs/reference/rust-api/
- Local IPC: https://casita.rs/integrations/ipc/
- Repository: https://github.com/cachix/casita
- Pinned revision: `90404fcb1cfb3d83f2233715448dfefe913f5fd1`, `main` on
  2026-09-30 ("Merge pull request #32 from cachix/docs/oci-import-guide").
  Source facts below were read from
  `https://raw.githubusercontent.com/cachix/casita/90404fcb1cfb3d83f2233715448dfefe913f5fd1/crates/casita/`.

Reviewed on 2026-09-30. Casita is pre-release and has no `v0.1.0` tag.

## Facts used by this change

- `crates/casita/Cargo.toml`: package `casita` 0.1.0, edition 2024,
  `rust-version = "1.94.1"`, Apache-2.0. The `native` feature enables
  `turso = "=0.8.0-pre.7"` from `https://github.com/cachix/turso.git` at
  `dca55133caa690f90dcdd58d3c4329fb0703659c`, `object_store` 0.14,
  `fastcdc` 5, `bao-tree` 0.16, `iroh-io` 0.6, `cap-std` 4, `nix-archive`
  0.6.0, and `astral-tokio-tar = "=0.6.4"`. `blake3 = "1.8"`. Default features
  are `cli`. `experimental` exposes unstable backend, format, tuning, and
  transport APIs. `s3`, `ssh`, `git`, and `oci` are opt-in.
- `src/import.rs`: `FilesystemImport`, `NarImport`, and `FilesystemNarImport`
  are public with `native`. `UnrootedFilesystemImport` and
  `MultiRootFilesystemImport` require `experimental`.
- `src/importers/filesystem.rs`: `UnrootedFilesystemImport` captures a
  directory "without creating a named root, within an existing session". "The
  session protects the imported graph for its lifetime." A repository cannot
  import it directly.
- `src/repository/mutation.rs`: `RootExpectation` (line 12),
  `ConditionalPublishResult` with a `RootMismatch` variant (lines 21-25),
  `MutationSession` (line 242), `MutationSession::import` (line 633), and
  `MutationSession::publish_if_roots_match` (line 730). Its doc comment states
  that nothing is committed on `RootMismatch`.
- Reported by the Mantle architecture worker from the same revision:
  `casita::experimental::{RootExpectation, RootChange, ConditionalPublishResult}`,
  `RootChange::Set { name, target }`, retries of unrelated revision races, and
  several expectations and changes publishing in one revision. `CasitarImport`
  defaults to requiring absent destinations (`src/importers/casitar.rs`).
- `src/nar.rs`: `VerifiedNarReport` holds locally measured facts and a
  retention hold, with no public constructor. `ensure_nar` measures missing
  facts or reuses local association facts; `scrub_nar` forces a full content
  audit.
- Rust API reference: `root`, `roots`, `set_root`, `compare_and_set_root`,
  `remove_root` (exact expected target), `retained_reader`,
  `preview_collection`, `collect`, `try_collect`, `fsck`, and `flush`. Local
  roots are permanent by default.
- Garbage collection guide: collection keeps every named root's closure and
  active pins. The local disk-pressure pass releases only roots marked
  evictable. Manual `gc` and `vacuum` keep every named root.
- Responsibilities page: the application decides why a graph matters and when
  to repoint or remove its root. Casita owns publication, verification, and
  reachability-based collection.
- IPC guide: the daemon exposes import, restore, and checkout only, without
  root listing, removal, blob access, or collection.

## Adaptation boundary

Mantle links Casita as a library inside `crunch-store` with `native` and
`experimental`. It stages envelopes in a mutation session, publishes roots
with conditional publication, and keeps admission, trust, retention, and GC
planning authority. Mantle does not adopt Casita network profiles, IPC, CLI,
FUSE, Git, or OCI features.

## Non-claims

This review records upstream source and documentation facts at one revision.
It does not prove Casita correctness, durability, performance, the stability
of experimental APIs, or the behavior of later revisions.
