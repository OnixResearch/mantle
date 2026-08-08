# Design: Split filesystem and castore NAR ownership

## Context

Mantle uses NAR in two different domains.

The filesystem domain starts from a host path. It needs byte-safe traversal, canonical entry order, executable-mode handling, explicit case-hack behavior, and streaming hash computation.

The castore domain starts from `snix_castore::Node` plus blob and directory services. It needs asynchronous reads, streaming NAR ingest, node reconstruction, and service-backed rendering.

`nix-archive` fits the filesystem domain. Adapted Snix fits the castore domain. Treating either library as a complete replacement would hide a real API and authority mismatch.

## Goals

- Use `nix-archive` for selected production filesystem NAR observations.
- Remove temporary castore round trips where the caller needs only NAR bytes or hashes.
- Preserve exact Nix-compatible bytes, size, and requested digest behavior.
- Keep asynchronous castore work on the existing Snix path.
- Detect drift before each production seam moves.
- Keep all filesystem work in a thin blocking shell.

## Decisions

### Decision 1: Pin the reviewed upstream source

**Choice:** Add exact package version `0.1.0`. Record upstream commit `14362ab589daa4869bda744d4fbe26a1914b5491`, license, reviewed APIs, Cargo checksum, and source-parity evidence.

The implementation must update `Cargo.lock` through Cargo. It must refresh `vendor-deps/` through the repository-owned dependency process.

**Rationale:** A moving dependency cannot support durable parity claims or offline self-builds.

### Decision 2: Add one shared Mantle adapter

**Choice:** Add a small shared crate for filesystem NAR work. The crate owns normalized requests, digest writers, byte counts, explicit case-hack selection, error mapping, and parity decisions.

Pure functions compare expected and observed NAR facts. A thin shell runs filesystem encoding and hashing in a blocking worker.

Callers must not import `nix_archive::nar` directly outside this adapter and its compatibility tests.

**Rationale:** One adapter prevents different store and project paths from selecting different defaults or error rules.

### Decision 3: Adopt filesystem encoding and hashing first

**Choice:** Use `nix-archive` encoding for selected host-path NAR observations. The first production seams are physical store verification and recursive project source hashing.

Use the dedicated upstream hash API for SHA-256. Use the upstream encoder with Mantle digest writers for other supported Nix hash algorithms.

Flat file hashes do not use NAR and remain unchanged.

**Rationale:** These paths currently need NAR facts from a filesystem tree. They do not need a castore node as their result.

### Decision 4: Keep castore NAR work on Snix

**Choice:** Keep `write_nar`, `SimpleRenderer`, and `ingest_nar_and_hash` for castore-backed rendering and ingest.

This includes native archive payloads, Nario payloads, HTTP cache imports, shared Rust cache transfer, remote build transfer, PathInfo repair, and output persistence.

**Rationale:** The upstream crate does not expose an asynchronous service-backed castore API.

### Decision 5: Require parity before migration

**Choice:** Each moved production seam needs a parity matrix before cutover.

The matrix covers empty files, regular files, executable files, symlinks, nested directories, sorted byte names, non-UTF-8 names, malformed trees, race attempts, and explicit case-hack modes.

Fixtures compare NAR bytes, size, SHA-256, and requested additional hashes. Nix differential tests run when `nix-store` is available.

A mismatch blocks the seam. The adapter cannot silently fall back to another implementation after a mismatch.

**Rationale:** Agreement must precede authority transfer.

### Decision 6: Make race and snapshot claims narrow

**Choice:** Record that descriptor-relative traversal resists path substitution during observation. Do not claim that one NAR hash is an atomic snapshot of a changing tree.

Callers that need stable source identity must keep their existing staging, revalidation, or mutation-lock rules.

**Rationale:** Safe traversal and atomic snapshots are different properties.

### Decision 7: Defer production decode and restore

**Choice:** Use borrowed decode only in bounded tests and fixture inspection during this change.

Do not use `restore_path` in production. The current API accepts a complete byte slice, and restoration can leave a partial destination after failure.

A future restore change must add explicit payload limits, fresh staging, cleanup, no-replace publication, and post-publication verification.

**Rationale:** Buffering and partial mutation conflict with current transport and publication requirements.

### Decision 8: Keep Nario coordination explicit

**Choice:** The active Nario v2 change can use `nix-archive` as a bounded compatibility oracle. It cannot route production Nario payload ingest through full-buffer decode.

Nario framing, signatures, PathInfo admission, rollback, and source projection remain outside this adapter.

**Rationale:** NAR byte compatibility does not own archive framing or trust.

## Failure Semantics

- An unsupported platform fails at compile or package selection before runtime work.
- An unsupported hash algorithm fails before filesystem traversal.
- A case-hack policy mismatch fails before a successful observation is returned.
- A filesystem read or mutation race returns a typed observation error.
- A parity mismatch blocks cutover and records both observations.
- A blocking worker failure returns through the caller's existing error type.
- No failure can create PathInfo, source locks, or successful archive receipts.

## Validation Strategy

- Run focused Snix, store, and project tests before implementation.
- Retain upstream goldens with exact source identity and license metadata.
- Add positive and negative adapter tests.
- Add generated-tree parity tests and explicit mutation-race tests.
- Run Nix differential tests when the oracle is available.
- Prove offline Cargo metadata with the checked vendor directory.
- Run package checks, Cairn gates, Tracey coverage, and relevant Nix checks.

## Non-Claims

- This change does not prove either NAR implementation correct.
- It does not prove atomic filesystem snapshots.
- It does not replace castore, PathInfo, Nario, cache, or publication authority.
- It does not prove package correctness, reproducibility, or release eligibility.
