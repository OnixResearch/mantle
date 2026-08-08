# ADR 0071: Adopt nix-archive at the filesystem NAR boundary

## Status

Accepted (2026-08-08)

## Context

Mantle needs Nix Archive (NAR) behavior in two domains.

Filesystem callers start from host paths. They need byte-safe traversal, canonical encoding, executable modes, case-hack policy, and streaming hashes.

Castore callers start from service-backed nodes. They need asynchronous rendering, streaming ingest, node reconstruction, and blob and directory service access.

Mantle currently uses adapted Snix for both domains. Some filesystem-only hash paths first ingest a tree into temporary castore services, then render that node back into NAR.

`cachix/nix-archive` version `0.1.0` provides byte-safe filesystem NAR encoding and hashing. It also provides borrowed decode and restore APIs over complete byte slices.

## Decision Drivers

- Preserve raw Unix filename and symlink bytes.
- Stream regular-file payloads during filesystem encoding.
- Remove temporary castore round trips from filesystem-only hash paths.
- Keep large archive and cache payloads on bounded streaming paths.
- Preserve Mantle's castore, PathInfo, trust, and evidence authority.
- Pin reviewed dependency and fixture identities for offline builds.
- Block cutover when implementations disagree.

## Decision

Mantle adopts `nix-archive` behind one shared adapter for selected filesystem NAR encoding and hashing.

The first production candidates are physical store verification and recursive project source hashing. Each candidate must pass current parity evidence before cutover.

Adapted Snix retains asynchronous castore NAR rendering and ingest. This includes native archives, Nario, HTTP caches, remote builds, shared Rust cache transfer, repair, and output persistence.

The adapter owns explicit case-hack selection, supported hash algorithms, byte counts, error mapping, and evidence fields. Filesystem work runs in a blocking shell. Pure functions compare expected and observed facts.

Production code cannot use full-buffer decode or restore in this change. A future restore change must add named limits, fresh staging, cleanup, no-replace publication, and post-publication verification.

The dependency uses exact package version `0.1.0`. Review evidence binds upstream commit `14362ab589daa4869bda744d4fbe26a1914b5491`. Implementation must prove that the packaged source matches the reviewed source.

## Alternatives Considered

### Replace all Snix NAR code

Rejected because `nix-archive` does not expose Mantle's asynchronous castore service model.

### Use nix-archive only as a test oracle

Rejected as the final design because it does not improve a production filesystem seam. The oracle role remains part of parity evidence.

### Restore NAR payloads directly to final paths

Deferred because restore accepts a complete byte slice and can leave a partial destination after failure.

### Keep temporary castore round trips

Rejected for filesystem-only observations after parity passes. Those round trips create work and duplicate the domain boundary.

## Consequences

- Mantle gains a second NAR implementation with a narrow owner boundary.
- Dependency and fixture identities become part of lifecycle evidence.
- Filesystem NAR observations can avoid temporary castore services.
- Castore transport behavior remains on Snix.
- Direct upstream imports need a dependency guard.
- Unsupported platforms fail at an explicit package boundary.
- Parity proves agreement on selected cases only. It does not prove either implementation correct.
- Descriptor-relative traversal reduces path-substitution risk. It does not prove an atomic tree snapshot.
