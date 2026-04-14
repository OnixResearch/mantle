# Tasks: Reduce build pipeline overhead

## PR 1: Direct derivation extraction helper

- [x] Add a direct typed derivation-extraction helper in `crunch-eval` for single-derivation and package-set Nickel results
- [x] Add regression tests for nested derivation inputs with Nickel enum tags through the direct path
- [ ] Confirm the current build path still passes unchanged while the helper is staged

## PR 2: Switch pipeline build path off JSON

- [x] Switch `crunch-pipeline::build()` from JSON-based derivation extraction to the new direct path
- [x] Keep `crunch eval` and other debug/reporting paths on explicit JSON export
- [x] Add pipeline tests proving single-root, package-set, and partial-failure behavior stays unchanged

## PR 3: Session source-resolution cache

- [ ] Add per-build-session memoization for source closure expansion
- [ ] Reuse previously known castore nodes or `PathInfo.node` values before re-ingesting the same on-disk tree
- [ ] Add regression tests showing repeated source inputs are resolved once per build session

## PR 4: Metadata-only remote closure lookup

- [ ] Add a metadata-only remote narinfo lookup path for closure walking
- [ ] Keep full NAR download and ingest on actual substitution only
- [ ] Add focused tests for metadata-only closure walks, misses, and graceful fallback

## PR 5: Worker hot-path ownership cleanup

- [ ] Reduce `BuildRequest` cloning on the dispatch path
- [ ] Reduce waiter-vector cloning on completion and failure propagation
- [ ] Reduce `Derivation` cloning on the ready/dispatch path if the first two cleanup steps leave meaningful remaining churn
- [ ] Add focused tests for the updated worker ownership flow

## Validation

- [x] Re-read the touched pipeline, nickel-eval, build-pipeline, and store specs against the final implementation plan before coding
- [x] Run `openspec validate reduce-build-pipeline-overhead`
