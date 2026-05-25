## Phase 1: Native Rust registry/vendored source planning

- [ ] [serial] r[rust_package_planning.native_registry_source.lockfile_identity] Parse and record supported registry package identities from `Cargo.lock`, including package coordinates, source URL/class, checksum material, and lockfile digest.
- [ ] [serial] r[rust_package_planning.native_registry_source.vendor_digest] Bind supported registry packages to declared vendor/source roots and deterministic BLAKE3 source-tree digests without reading ambient Cargo caches.
- [ ] [serial] r[rust_package_planning.native_registry_source.oracle_compare] Compare supported native registry source facts with retained Cargo oracle material and emit deterministic mismatch blockers.
- [ ] [serial] r[rust_package_planning.native_registry_source.blockers] Fail closed for missing checksums, missing/unreadable vendor roots, digest mismatches, unsupported source kinds/layouts, and any undeclared source-cache dependency.
- [ ] [serial] r[rust_package_planning.native_registry_source.receipts] Preserve JSON receipt evidence that identifies ready registry source facts, source digests, blockers, and the bounded non-network/non-resolver claim.
- [ ] [serial] r[rust_package_planning.native_registry_source.tests] Add focused positive and negative Rust planner/CLI fixtures for vendored registry source readiness and stale/missing source blockers.
- [ ] [serial] r[rust_package_planning.native_registry_source.verify] Run focused Rust verification, Cairn validation, and proposal/design/tasks gates before implementation closeout.
