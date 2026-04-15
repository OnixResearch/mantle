# Tasks: Contain tarball extraction

## Phase 1: Containment helpers

- [ ] Add pure helper coverage in `crates/crunch-build/src/fetcher.rs` for
      stripped entry paths and link targets that must stay within the requested
      output tree
- [ ] Add parent-chain validation so later file or directory writes cannot walk
      through an extracted symlink that points outside the output tree
- [ ] Keep valid in-tree paths and links working for normal tarballs

## Phase 2: Extraction enforcement

- [ ] Apply the containment helpers to regular-file, directory, symlink, and
      hardlink extraction paths
- [ ] Return clear `FetchError::TarError` messages that name the offending
      archive path or link target
- [ ] Preserve top-level prefix stripping while rejecting post-strip escape
      paths such as `pkg/../../escape`

## Phase 3: Regression coverage and validation

- [ ] Add crafted tar tests for path traversal rejection
- [ ] Add crafted tar tests for absolute-symlink rejection,
      symlink-parent escape rejection, and valid in-tree symlink preservation
- [ ] Add crafted tar tests for hardlink-target escape rejection
- [ ] Run `openspec validate contain-tarball-extraction`
- [ ] Run `cargo test -p crunch -p crunch-pipeline --lib --tests` under the
      documented Cargo environment
