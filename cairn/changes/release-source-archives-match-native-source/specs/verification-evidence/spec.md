# Verification Evidence Specification Delta

## ADDED Requirements

### Requirement: Release source archives preserve native source identity [r[verification_evidence.release_source_archive_native_parity]]

Mantle release evidence source archives MUST preserve the tracked source files that native path-source hashing observes for provider-bound release proofs, while excluding private runtime and lifecycle evidence paths that native source hashing skips.

#### Scenario: tracked package source is replayable

- GIVEN `mantle release create` packages a release source archive from a checkout
- WHEN a witness extracts the source archive and runs a provider fixed-point replay
- THEN the extracted tree MUST contain tracked package-root files, scripts, docs, tests, and verified vendored source files visible to native path-source hashing
- AND the archive MUST NOT reduce the source tree to only the self-build staging allowlist.

#### Scenario: private and skipped paths stay out of source archives

- GIVEN the checkout contains untracked files, ignored runtime output directories, private signing material, or root Cairn lifecycle evidence
- WHEN `mantle release create` packages a release source archive
- THEN the archive MUST exclude those paths before copying source material into a release bundle
- AND exclusion MUST apply even if a private runtime path such as `target/` or `.pi/` is force-tracked.
