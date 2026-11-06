# ADR 0141: Bind successor resolution to the consuming workspace

## Status

Accepted 2026-11-06 for Cargo-free successor source-topology authoring. This is
not acceptance of source preparation, a provider, or a fixed-point proof.

## Context

The current consumer is Rust successor graph authoring for the source-built
cold route. Real source preparation reached `original vendor identity memchr
2.7.1 is not unique`. Inspection of its authenticated Rust 1.90 archive found
no vendored memchr 2.7.1: the consuming root and library locks pin 2.7.5, while
vendored bstr 1.12.0 carries its independent upstream lock pinning 2.7.1.

The successor resolver searched for a lockfile from each dependency's own
manifest. That imported dependency-local lock authority into the consuming
build. It also searched for patches from each dependency's workspace, although
registry patches belong to the consuming resolution root. Compiler, standard
library, Cargo and rustdoc goal groups are authored as separate topologies.

## Decision Drivers

- Preserve the exact locked identities and local patch declarations of the
  consuming source workspace.
- Never substitute a dependency-local or parent-workspace lock when the
  consuming lock is missing.
- Reuse the existing package keys, graph traversal and feature propagation.
- Avoid duplicating a workspace string or lock table in every package key.
- Keep source-only, no-fetch, no-fallback admission and existing resource bounds.

## Decision

Derive one borrowed resolution context from each topology's goals. Goals must
share the same consuming workspace, or the standalone package root when no
workspace is declared. Reject a mixed-workspace topology rather than merge
conflicting lock/patch authorities. Empty goal sets retain their empty topology.

Look up only that context's Cargo.lock. A missing lock remains an error when a
registry edge requires it; a path-only graph does not need an invented lock.
Registry selection and transitive patch lookup use this context throughout the
walk, including host/build dependencies and vendored crates with their own
workspace and lockfile. Local patch paths are relative to the consuming root.

Manifest workspace inheritance still uses each package's declared workspace;
it is separate from the global lock/patch authority. Package keys and caller
APIs are unchanged. No source/version fallback, additional fetching, or guessed
vendor choice is introduced.

## Consequences

Mantle's Rust source-authoring owners maintain this behavior. Separate topology
calls preserve distinct compiler/library/Cargo lock scopes without expanding
every graph key. A future multi-workspace operation must explicitly author
separate topologies rather than accidentally share one feature/identity graph.

Behavioral regressions cover stale dependency locks, missing consuming locks,
nested consuming workspaces, transitive patches and nonlocal patch rejection,
and mixed-root denial. Real preparation against the original authenticated
archive remains required evidence; synthetic fixtures and a prepared profile
cannot establish provider execution or bootstrap completion.
