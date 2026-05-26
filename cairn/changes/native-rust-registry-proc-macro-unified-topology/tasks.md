# Tasks: Native Rust registry proc-macro unified topology

- [ ] [serial] r[rust_package_planning.native_registry_proc_macro_unified_topology] Add accepted requirement coverage for bounded vendored-registry proc-macro unified topology execution and non-goals.
- [ ] [serial] r[rust_package_planning.native_registry_proc_macro_unified_topology] Gate registry-backed proc-macro execution on ready native registry source facts, ready native host-unit graph facts, and ready unit derivation graph evidence.
- [ ] [serial] r[rust_package_planning.native_registry_proc_macro_unified_topology] Ensure unified topology schedules supported registry proc-macro host producers before target consumers and binds produced host artifacts into downstream `rustc` material.
- [ ] [serial] r[rust_package_planning.native_registry_proc_macro_unified_topology] Emit deterministic pre-`rustc` blockers for missing, stale, unsupported, or ambient-cache-dependent registry proc-macro source/host material.
- [ ] [serial] r[rust_package_planning.native_registry_proc_macro_unified_topology] Add positive `rust_plan_cli` JSON coverage for a local target consuming a vendored registry proc-macro through `--execute-topology`.
- [ ] [serial] r[rust_package_planning.native_registry_proc_macro_unified_topology] Add negative `rust_plan_cli` JSON coverage proving unsupported/missing vendored registry proc-macro material blocks before any proc-macro/target execution.
- [ ] [serial] r[rust_package_planning.native_registry_proc_macro_unified_topology] Run focused Rust verification, Cairn validation/gates, sync/archive the change, verify accepted spec content, commit, push, and confirm clean/synced.
