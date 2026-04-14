# Build Pipeline Specification

## Purpose

Defines how derivations are built, cached, logged, and persisted.

## ADDED Requirements

### Requirement: Build session reuses source preparation work

The build pipeline MUST memoize source-closure expansion and reusable
input-node materialization within a single build session. Repeating the same
source path in multiple derivations MUST reuse the previously resolved closure
member set and previously known castore node when that node is still valid.

#### Scenario: Shared source closure resolved once per session

- GIVEN two derivations in one build session depend on the same source input
  path
- WHEN the first derivation resolves that source path's runtime closure
- THEN the second derivation reuses the memoized closure member set
- AND the pipeline does not repeat a fresh transitive closure walk for that
  same source path

#### Scenario: Existing local node reused without disk re-ingest

- GIVEN a dependency output already has local `PathInfo` and castore content
- WHEN a later derivation in the same session needs that output as a sandbox
  input
- THEN the pipeline reuses the known node metadata
- AND it does not re-ingest the same filesystem tree from disk before mounting
  it
