## ADDED Requirements

### Requirement: Tarball extraction containment

Tarball extraction MUST keep every extracted path inside the requested output tree.

The tarball fetcher MUST apply containment checks after top-level directory
stripping and before any filesystem effect for regular files, directories,
symlinks, and hardlinks.

If an archive entry path or effective link target would escape the requested
output tree, the fetcher MUST fail closed with a clear extraction error naming
the offending entry, and it MUST NOT continue extraction past that invalid
entry.

#### Scenario: Path traversal entry is rejected after prefix stripping

- GIVEN a tarball entry whose stripped path contains `..`, such as
  `pkg/../../escape`
- WHEN crunch extracts the tarball
- THEN extraction fails with a clear containment error
- AND crunch does not write the escaped path outside the requested output tree

#### Scenario: Absolute symlink target is rejected

- GIVEN a tarball symlink entry such as `pkg/link -> /etc/passwd`
- WHEN crunch extracts the tarball
- THEN extraction fails with a clear containment error
- AND crunch does not materialize a link that points outside the requested
  output tree

#### Scenario: Later write through escaped symlink parent is rejected

- GIVEN a tarball that first creates `pkg/out` as a symlink to `../../escape`
- AND a later tarball entry writes `pkg/out/file.txt`
- WHEN crunch extracts the tarball
- THEN extraction fails with a clear containment error
- AND crunch does not write `file.txt` outside the requested output tree

#### Scenario: Hardlink target escape is rejected

- GIVEN a tarball hardlink entry whose effective target resolves outside the
  requested output tree
- WHEN crunch extracts the tarball
- THEN extraction fails with a clear containment error
- AND crunch does not materialize the hardlink target outside the requested
  output tree

#### Scenario: Valid in-tree link remains allowed

- GIVEN a tarball symlink or hardlink entry whose effective target stays within
  the requested output tree
- WHEN crunch extracts the tarball
- THEN extraction succeeds
- AND the in-tree link behavior is preserved
