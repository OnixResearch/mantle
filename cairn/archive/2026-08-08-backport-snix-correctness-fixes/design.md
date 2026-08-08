# Design: Selective Snix correctness backports

## Context

Mantle’s vendored Snix crates are an adapted fork. The review in `evidence/upstream-review.md` separates selected behavior from deferred refactors and incompatible serialization changes.

The selected work spans four boundaries:

1. remote cache request identity and transport handling;
2. castore size and FUSE metadata;
3. redb and overlay-service behavior;
4. filesystem ingestion and tracing filters.

## Goals

- Close the confirmed remote cache identity gap before any local mutation.
- Backport small correctness fixes without broad API churn.
- Keep pure comparisons and mappings separate from I/O shells.
- Preserve Mantle-specific identity and store behavior.
- Prove each selected behavior with positive and negative tests.

## Decisions

### Decision 1: Keep an explicit upstream disposition ledger

**Choice:** Every reviewed Gerrit change receives an adopted, adapted, deferred, or rejected disposition in `evidence/upstream-review.md`.

An adopted or adapted entry names the local files, required local differences, and test tasks. A deferred entry names a concrete reopen condition. A rejected entry names the incompatible Mantle boundary.

**Rationale:** The vendor tree has no single upstream base. A behavior ledger is more accurate than an inferred revision range.

### Decision 2: Enforce requested PathInfo identity twice

**Choice:** Add a pure comparator over the requested digest and returned store-path digest. The Snix HTTP service must reject a mismatch before it returns `PathInfo` to its caller.

Mantle’s remote-substitution shell must apply the same decision before it persists PathInfo or performs any later side effect. This second guard covers alternative `PathInfoService` implementations and test services.

A mismatch is an integrity error, not a cache miss. The diagnostic must identify the mismatch class without exposing secret URL or trust configuration.

The guard must run before:

- local PathInfo persistence;
- artifact-attestation creation;
- castore or output-node registration;
- physical output export;
- retained-root registration;
- advisory hit publication;
- successful substitution reporting.

**Rationale:** Signed metadata proves the returned path. It does not prove that the returned path matches the request.

### Decision 3: Normalize cache transports before decode or join

**Choice:** Configure zstd decoders to consume all concatenated frames. A later truncated or malformed frame must fail the whole import or substitution path.

Normalize a binary-cache base URL as a directory before `Url::join`. The normalization must preserve the configured path and existing credential-rejection rules. Equivalent bases with and without a trailing slash must resolve the same narinfo and NAR endpoints.

Implement URL normalization and expected-endpoint derivation as pure functions. Keep HTTP reads and decoder I/O in thin shells.

**Rationale:** The default zstd decoder can stop after one valid frame. `Url::join` can also replace the last path segment when the base lacks a trailing slash.

### Decision 4: Make castore metadata mappings exact

**Choice:** Repair `Directory::size()` so each node contribution is counted once. Keep the computation pure and use checked arithmetic or the existing bounded representation.

Map castore node kinds to FUSE `DT_*` directory-entry values. Do not use inode mode `S_IF*` values for `readdir` entry types.

Set a valid nonzero link count in FUSE attributes according to the adopted upstream behavior. Add tests for directories, regular files, and symlinks.

**Rationale:** Size accounting and FUSE metadata are observable filesystem semantics. Similar numeric constants from different kernel interfaces are not interchangeable.

### Decision 5: Keep blocking database work inside the blocking worker

**Choice:** Move redb write-transaction creation and use into `spawn_blocking`. Move an owned `Arc` of the database into that closure.

Do not hold a redb transaction across an async scheduling boundary. Keep record validation and key derivation in pure helpers where practical.

**Rationale:** Transaction creation can block. Starting it on the async executor can stall unrelated work.

### Decision 6: List only the writable near PathInfo service

**Choice:** `PathInfoCache::list()` delegates to the writable near service. It does not enumerate the remote or read-through far service.

Writes already target the near service. Local store mutation commands, including signing, need the same local listing authority. Far-only records remain discoverable through keyed reads and do not become local mutation candidates through listing.

**Rationale:** Listing the far service can turn remote discovery into unintended local mutation scope.

### Decision 7: Adopt bounded operational fixes without API refactors

**Choice:** Use the upstream filesystem-ingestion copy path based on the maintained default buffer instead of a fixed oversized buffer.

Apply the tracing `EnvFilter` to the combined subscriber layers. Filtering only the formatting layer must not suppress required non-format layers.

Do not adopt the open concurrent-uploader, derivation-builder, output-model, or combined-store-trait refactors in this change.

**Rationale:** These small fixes improve behavior without changing Mantle’s public architecture.

### Decision 8: Keep conditional fixes deferred until their trigger is active

**Choice:** Keep the virtiofs used-length fix deferred until Mantle activates that queue path. Keep read-only redb builder configuration deferred until Mantle sets non-default builder options on read-only opens.

The implementation review must confirm those conditions again. If either condition is active, the relevant fix and tests enter this change before completion.

**Rationale:** Conditional code changes need a reachable Mantle behavior or a separate compatibility goal.

## Failure Semantics

- A remote PathInfo request mismatch returns an integrity error before mutation.
- A malformed later zstd frame rejects the complete payload.
- A cache base that cannot become a valid directory URL fails before network access.
- Invalid FUSE node-type mapping cannot emit a successful directory entry.
- A redb blocking-task failure returns through the existing typed service error.
- A far-service listing failure cannot affect near-service listing because the far list is not called.

## Validation Strategy

- Run focused vendor and `crunch-store` tests before implementation for a baseline.
- Add positive and negative tests beside each selected behavior.
- Use a panic-on-call far service to prove cache listing remains near-only.
- Use a mutation-counting local service to prove mismatched remote PathInfo causes zero writes and zero side effects.
- Use concatenated and truncated zstd fixtures.
- Run package checks, formatting, Cairn validation, proposal/design/tasks gates, and Tracey coverage.

## Non-Claims

- This change does not prove full parity with upstream Snix.
- It does not certify open Gerrit changes as accepted upstream.
- It does not change Mantle’s postcard directory identity.
- It does not prove arbitrary FUSE, redb, HTTP, or compression correctness.
- It does not establish release eligibility by itself.
