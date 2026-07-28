# Tasks

## Phase 1: Architecture and pure identity

- [x] [serial] I1 Add ADR 0035 for local-first Rust unit caching, separate action-result authority, atomic materialization, and deferred wrapper integration. r[rust_package_planning.unit_execution.topology.castore_result_cache]
  - Evidence: `adr/0035-cache-rust-units-through-castore-action-results.md` records the proposed planning boundary.
- [ ] [serial] I2 Add `crunch-rust-cache-core` with bounded versioned action and result schemas, canonical BLAKE3 identities, candidate classification, and no filesystem or process dependencies. r[rust_package_planning.unit_execution.topology.castore_result_cache]
- [ ] [serial] I3 Bind compiler content, compiler version, sysroot or provider closure, platform, source, arguments, admitted environment, dependencies, host artifacts, build-script facts, native-link facts, and policy identity into the action reference. r[rust_package_planning.unit_execution.topology.castore_result_cache.identity]
- [ ] [parallel] I4 Add positive and negative core tests for deterministic identity, path normalization, every invalidation class, bounds, malformed records, and conflicting candidates. r[rust_package_planning.unit_execution.topology.castore_result_cache.identity]

## Phase 2: Local storage and materialization shell

- [ ] [serial] I5 Add `crunch-rust-cache` as the thin shell for local result indexes, castore ingestion, completeness probes, and result publication. r[rust_package_planning.unit_execution.topology.castore_result_cache.local_reuse]
- [ ] [serial] I6 Add verified sibling-directory materialization with create-new staging, bounded export, artifact verification, atomic commit, and cleanup after every failure. r[rust_package_planning.unit_execution.topology.castore_result_cache.atomic_materialization]
- [ ] [serial] I7 Exclude per-run execution receipts from immutable result trees and emit a fresh receipt after every restored or compiled unit result. r[rust_package_planning.unit_execution.topology.castore_result_cache.local_reuse]
- [ ] [serial] I8 Add retention and garbage-collection handling for indexed Rust unit result roots. r[rust_package_planning.unit_execution.topology.castore_result_cache.retention]
- [ ] [parallel] I9 Add negative tests for missing blobs, missing directories, bad digests, path escapes, symlink surprises, over-limit trees, conflicting results, interrupted export, and cross-filesystem commit. r[rust_package_planning.unit_execution.topology.castore_result_cache.atomic_materialization]

## Phase 3: Rust-plan integration

- [ ] [serial] I10 Wire explicit local cache policy and the resolved Mantle state directory into Rust topology execution without reading ambient Cargo cache state. r[rust_package_planning.unit_execution.topology.castore_result_cache.local_reuse]
- [ ] [serial] I11 Preserve existing execution-directory reuse first, query castore second, and invoke `rustc` only after an admitted miss. r[rust_package_planning.unit_execution.topology.castore_result_cache.local_reuse]
- [ ] [serial] I12 Extend unit and topology receipts with stable local-hit, local-miss, rejection, conflict, restored-byte, reused-byte, and compiler-executed facts. r[rust_package_planning.unit_execution.topology.castore_result_cache.evidence]
- [ ] [parallel] I13 Add a positive integration test that deletes the prior execution root, restores every unit from castore, verifies artifacts, and observes zero compiler invocations. r[rust_package_planning.unit_execution.topology.castore_result_cache.local_reuse]
- [ ] [parallel] I14 Add invalidation tests for source, compiler, sysroot, target, profile, features, arguments, environment, dependency, proc-macro, build-script, native-link, and policy changes. r[rust_package_planning.unit_execution.topology.castore_result_cache.identity]
- [ ] [parallel] I15 Update operator and machine-artifact documentation with the cache schema, lookup order, failure classes, retention rule, atomic materialization boundary, and non-claims. r[rust_package_planning.unit_execution.topology.castore_result_cache.evidence]

## Phase 4: Performance and lifecycle evidence

- [ ] [serial] V1 Run `nix develop -c cargo test -p crunch-rust-cache-core`, focused `crunch-rust-cache` tests, focused `mantle` Rust cache tests, and the local zero-compiler-invocation integration rail. Record exact output in `cairn/changes/persist-rust-unit-castore-results/evidence/verification.md`. r[rust_package_planning.unit_execution.topology.castore_result_cache.evidence]
- [ ] [serial] V2 Benchmark cold compilation, existing output-directory reuse, castore restoration after output deletion, and cache-miss overhead with named sample and size limits. Record median and tail latency plus transferred and reused bytes. r[rust_package_planning.unit_execution.topology.castore_result_cache.performance]
- [ ] [serial] V3 Run `nix run path:/home/brittonr/git/OnixResearch/cairn#cairn -- validate --root .`, `gate proposal`, `gate design`, `gate tasks`, and `tracey coverage` for this change. Record exact output before sync and archive. r[rust_package_planning.unit_execution.topology.castore_result_cache.evidence]
