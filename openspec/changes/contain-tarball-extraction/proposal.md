# Contain tarball extraction

## Why

`crates/crunch-build/src/fetcher.rs` currently extracts tarball entries onto the
filesystem with incomplete containment checks. A crafted archive can still try
to escape the requested output tree through entry paths, link targets, or a
write that traverses a previously-created symlink parent.

That is too much trust for a network-facing fetch path. Hash verification helps
for honest inputs, but dummy hashes and `--fix` workflows still make fail-closed
extraction worthwhile.

## What Changes

- require tar extraction to keep every regular file, directory, symlink, and
  hardlink inside the requested output tree after top-level prefix stripping
- reject tar entries whose paths or link targets would escape containment,
  including writes that traverse an extracted symlink parent
- add regression tests with crafted archives for traversal and link-escape
  cases
- keep the change scoped to tar extraction hardening, not broader fetcher I/O

## Capabilities

### New Capabilities

- `tarball-extraction-containment`: tarball extraction fails closed on entries
  that would escape the requested output tree
- `tarball-link-containment`: tarball symlink and hardlink targets are checked
  against the extraction root before filesystem effects

## Impact

- **Files**: `crates/crunch-build/src/fetcher.rs` and tar-extraction tests in
  the same module
- **Behavior**: invalid tarballs fail with a clear extraction error instead of
  partially writing outside the output tree
- **Dependencies**: none
- **Testing**: add crafted tar regression tests, then rerun targeted fetcher
  tests and `cargo test -p crunch -p crunch-pipeline --lib --tests`

## Non-Goals

- change fetch download timeouts or retry policy
- redesign `FetchBuildService` or fixed-output hash verification
- add new archive formats beyond current tarball support
- change git fetch materialization behavior
