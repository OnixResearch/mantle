## Phase 1: Rust topology output reuse

- [x] [serial] r[rust_package_planning.topology_output_reuse.lookup] Add deterministic prior per-unit topology execution receipt lookup under the explicit execution output root.
- [x] [serial] r[rust_package_planning.topology_output_reuse.match] Reuse prior declared outputs only when current explicit unit identity, source closure digest, dependency artifact digests, host artifact digests, toolchain identity, rustc argument digest, declared output paths, and output BLAKE3 digests match prior receipt material.
- [x] [serial] r[rust_package_planning.topology_output_reuse.blockers] Add fail-closed stale-cache blockers for malformed prior receipts, missing/unreadable outputs, digest mismatches, or prior receipt material that no longer matches current explicit inputs.
- [x] [serial] r[rust_package_planning.topology_output_reuse.receipts] Preserve CLI JSON evidence that explains per-unit `rebuilt` versus `reused` decisions and keeps the bounded Cargo-free topology claim explicit.
- [x] [serial] r[rust_package_planning.topology_output_reuse.tests] Add focused positive and negative `rust_plan_cli` coverage for repeated-run reuse and stale cached output behavior.
- [x] [serial] r[rust_package_planning.topology_output_reuse.verify] Run focused Rust verification, Cairn validation, and proposal/design/tasks gates before implementation closeout.
