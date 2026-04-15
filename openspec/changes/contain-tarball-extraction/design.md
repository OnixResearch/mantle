# Design: Contain tarball extraction

## Context

Crunch already strips the top-level directory from fetched tarballs, but the
remaining extraction path still needs a stronger safety contract. The fetch path
must reject archives that try to escape through `..`, symlink targets,
hardlink targets, or later writes through a symlink parent created earlier in
the same archive.

This is a small, high-leverage hardening change in a network-facing path. The
clean boundary is `crates/crunch-build/src/fetcher.rs`.

## Goals / Non-Goals

**Goals:**

- make tar extraction fail closed when an entry would escape the requested
  output tree
- apply one containment policy to regular files, directories, symlinks, and
  hardlinks
- keep the hardening logic mostly pure and unit-testable
- add regression tests that reproduce the escape attempts directly

**Non-Goals:**

- change non-tar fetchers
- canonicalize against the live host filesystem outside the extraction root
- change download timeout policy or tempdir lifecycle

## Decisions

### 1. Containment is lexical and checked before filesystem effects

**Choice:** extract-tar path checks will use pure helpers that resolve stripped
entry paths and link targets relative to the intended extraction root without
calling `canonicalize()` on host paths.

**Rationale:** `canonicalize()` needs existing filesystem state, follows live
symlinks, and makes the result depend on what already exists on disk. Tar
containment is simpler and more deterministic as a lexical policy over archive
paths plus already-created in-tree symlinks.

**Implementation:** add helper(s) in `fetcher.rs` that normalize candidate
archive paths after prefix stripping and reject any resolution that escapes the
output root. This no-`canonicalize()` rule applies to entry-path normalization
itself; it does not forbid scoped checks against already-created paths inside
the extraction root.

### 2. Parent-path writes must refuse symlink redirection

**Choice:** before creating a directory or file below `dest`, extraction must
validate that each existing parent component inside the output tree is either a
real directory or a symlink whose resolved target still stays inside the output
root.

**Rationale:** rejecting raw `..` in the current entry path is not enough if the
archive first creates `pkg/out -> ../../escape` and later writes `pkg/out/file`.
The later write follows the symlink unless crunch checks the parent chain. That
check is intentionally scoped to the extraction tree crunch is creating, not to
general host-path canonicalization outside the output root.

**Implementation:** use the same containment helper for parent resolution before
`create_dir_all`, file creation, or copy-style hardlink materialization.

### 3. Link entries share one fail-closed policy

**Choice:** symlink and hardlink entries are valid only when their effective
archive target resolves inside the extraction root after top-level prefix
stripping. Otherwise extraction returns `FetchError::TarError` naming the
offending entry.

**Rationale:** symlink and hardlink entries are just alternate ways to redirect
writes or references. The fetcher should not have weaker rules for one link type
than the other.

**Implementation:** keep valid in-tree links working, reject escape targets, and
avoid partial best-effort extraction once an invalid link is seen.

### 4. Regression coverage uses crafted tar archives

**Choice:** tests will build tarballs in-memory with explicit headers for the
bad entries instead of relying on host `tar` behavior or prebuilt fixtures.

**Rationale:** crafted archives let the tests pin exact edge cases and keep the
coverage local to `fetcher.rs`.

**Implementation:** add tests for path traversal, absolute-symlink rejection,
symlink-parent escape, hardlink-target escape, and one valid in-tree symlink
case.

## Risks / Trade-offs

**Stricter extraction can reject archives that a permissive tool would unpack**
→ acceptable here; fail-closed is the point of the change.

**Path handling logic gets more intricate**
→ mitigate with small helpers and crafted tests for each archive entry type.
