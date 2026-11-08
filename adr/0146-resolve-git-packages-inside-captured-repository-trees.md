# ADR 0146: Resolve git-sourced packages inside captured repository trees

## Status

Accepted (2026-11-08) for the source-built fixed-point vendor input cohort and
native planning. This does not accept source preparation, provider execution, a
fixed point or a release.

## Context

The Cargo-free plan rejected the frozen vendor cohort with
`missing-path-dependency-manifest` for `vendor-deps/turso/../../core/Cargo.toml`
and `vendor-deps/basalt/vendor/ucan/Cargo.toml`.

Both are intra-repository path dependencies of git-sourced packages:

- the `turso` repository keeps `core/` (the `turso_core` member) beside its Rust
  bindings, so a member links to a sibling through `..`;
- `vendor-deps/basalt` carried only `vendor/ucan.snapshot` and
  `vendor/ucan-revision.txt`, while the pinned basalt revision contains the
  generated `vendor/ucan/**` tree.

A per-crate vendored layout cannot express either dependency: `cargo vendor`
copying each package separately flattens the repository, and any relative path
that leaves the package directory no longer names a file. The planner correctly
refused to invent the missing source instead of guessing a substitute.

## Decision

A git-sourced lock package resolves inside a captured repository tree when one
is present at `<vendor root>/git-repositories/<host>/<path>`, where `<path>` is
the lock source URL without its scheme, `?query` and `#fragment` and without a
trailing `.git`. The member manifest is the unique `Cargo.toml` in that tree that
declares the package name and its effective version, where an inherited
`version.workspace = true` resolves from the nearest ancestor workspace manifest.
Several matches or none leave the package unresolved; traversal order never
chooses. Per-crate vendored directories remain the fallback for registry-sourced
packages and for older git captures.

`scripts/capture-git-repositories.py` materialises those trees from
`Cargo.lock` at the locked revisions, preferring local repository databases and
fetching only when a revision is absent. It replaces an in-tree symlink whose
target contains `..` with a copy of the resolved in-tree content, because the
bundle's symlink contract admits no `..` component, and it rejects a link that
escapes the repository instead of capturing it.

`mantle source bundle refresh-vendor-inputs` replaces only the
`vendored-cargo-inputs` record of a verified source-built profile, preserving
every other record and its identity, using the same machinery that refreshes the
Mantle source record.

## Decision Drivers

- Source authority must remain the exact locked revision; a missing file is a
  blocker, never a reason to substitute another version or source.
- Repository layout carries meaning: sibling members and generated vendored
  trees are part of the source a build reads.
- The bundle's lexical symlink contract is not weakened for cohort convenience.

## Consequences

- The vendor input record carries complete repository trees, so it is larger
  than a per-crate mirror; the admitted payload bound and per-file chunking are
  unchanged.
- A cohort update that misses a repository revision fails planning with a named
  missing manifest instead of silently resolving a different file.
- Capturing runs outside the proof: fetching a repository is an operator action
  recorded in the capture receipt, and the proof itself remains offline.
- The transition, StageX lineage and native-provider pins are unaffected; the
  source-profile identity changes because one classified record changed.
