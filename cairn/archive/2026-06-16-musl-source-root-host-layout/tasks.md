## Phase 1: Source-root host layout support

- [x] [serial] I1 Add source-root-musl host layout classification and host member path mapping for musl-host Rust providers. r[rust_package_planning.source_built_toolchain_closure.native_materialization]
- [x] [serial] V1 [evidence=cairn/changes/musl-source-root-host-layout/evidence/musl-host-layout-focused-validation-2026-06-16.md] Run focused positive/negative tests, formatting, and diff checks. r[rust_package_planning.source_built_toolchain_closure.native_materialization]

## Phase 2: Current-provider guardrail

- [x] [serial] V2 [evidence=cairn/changes/musl-source-root-host-layout/evidence/current-gnu-host-rejection-2026-06-16.md] Rerun the current GNU-host provider frontier and prove source-root musl is still rejected as a GNU host root. r[rust_package_planning.source_built_toolchain_closure.native_materialization]
- [x] [serial] V3 [evidence=cairn/changes/musl-source-root-host-layout/evidence/archive-validation-2026-06-16.md] Archive and validate the completed musl-host layout slice. r[rust_package_planning.source_built_toolchain_closure.native_materialization]
