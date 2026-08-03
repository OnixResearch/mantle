# Tasks: Extract Mantle store decision cores

## Phase 1: Baseline and observation inventory

- [ ] [serial] V1 Run focused `crunch-store` GC and final-NAR repair tests before core changes and freeze candidate plans, reports, canonical bytes, digest roles, signatures, sidecar dispositions, and mutation order. r[mantle.store_lifecycle.compatibility]
- [ ] [serial] I1 Inventory every GC live-root and retention source, including project, closure, action-result, Rust unit result, sidecar, and active-change authority. r[mantle.store_lifecycle.gc_core]
- [ ] [serial] I2 Inventory repair observations and separate planning, execution, rollback, and post-write verification authority. r[mantle.store_lifecycle.repair_core]
- [ ] [parallel] I3 Define bounded normalized observation, intent, report, digest-role, and error DTOs for both cores. r[mantle.store_lifecycle.core_boundary]

## Phase 2: Repair core

- [ ] [serial] I4 Add alloc-only `crunch-repair-core` and move final-NAR repair admission, no-op, rejection, signature disposition, sidecar disposition, and report decisions into it. r[mantle.store_lifecycle.repair_core]
- [ ] [serial] I5 Keep PathInfo reads, NAR rendering, SHA-256 calculation, key access, signing, staging, persistence, rollback, cleanup, and verification in `crunch-store` adapters. r[mantle.store_lifecycle.core_boundary]
- [ ] [parallel] V2 Add positive current and repairable fixtures plus negative selector, identity, digest, signature, sidecar, rollback, and verification fixtures. r[mantle.store_lifecycle.repair_core] r[mantle.store_lifecycle.compatibility]

## Phase 3: GC core

- [ ] [serial] I6 Add alloc-only `crunch-gc-core` for reachability, retention, live/dead classification, orphan admission, reclaim summaries, ordered intents, and dry-run reports over supplied observations. r[mantle.store_lifecycle.gc_core]
- [ ] [serial] I7 Keep service queries, directory and metadata scans, size observation, locking, deletion, database replacement, and cleanup in `crunch-store` adapters. r[mantle.store_lifecycle.core_boundary]
- [ ] [parallel] V3 Add positive retained-closure, shared-object, dry-run, and deterministic-order fixtures plus negative missing-root, cycle, stale-retention, overflow, path-role, incomplete-observation, and active-result-reference fixtures. r[mantle.store_lifecycle.gc_core] r[mantle.store_lifecycle.compatibility]

## Phase 4: Compatibility and enforcement

- [ ] [serial] I8 Preserve accepted CLI behavior, canonical reports, Nix SHA-256 fields, BLAKE3 identities, store paths, signatures, sidecars, candidate sets, and mutation order through explicit adapters. r[mantle.store_lifecycle.compatibility]
- [ ] [serial] I9 Add both cores to the maintained no-std inventory, ownership review, Octet topology, API-shape checks, and host plus `wasm32-unknown-unknown` checks. r[mantle.store_lifecycle.architecture_guard]
- [ ] [parallel] V4 Add positive shell-boundary fixtures and negative filesystem, process, environment, clock, network, async, store-handle, PathInfo-service, and cross-core dependency fixtures. r[mantle.store_lifecycle.architecture_guard]
- [ ] [serial] I10 Document observation ownership, hash roles, adapter authority, mutation non-claims, and independent mechanism scope. r[mantle.store_lifecycle.claim_boundary]

## Phase 5: Closeout

- [ ] [serial] V5 Run focused core and `crunch-store` tests, both wasm checks, `./scripts/check-no-std-core.sh`, first-party Tiger Style and quality rails, Cairn validation, proposal/design/tasks gates, and the relevant Nix checks. Record exact results before sync and archive. r[mantle.store_lifecycle.architecture_guard] r[mantle.store_lifecycle.compatibility]
