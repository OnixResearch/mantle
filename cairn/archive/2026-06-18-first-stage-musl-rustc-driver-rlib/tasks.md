# Tasks

- [x] [serial] I1 Add a musl-host-only first-stage source patch that rewrites `compiler/rustc_driver/Cargo.toml` from `dylib` to `rlib` and fails closed on unexpected source shape. r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_rustc_driver_rlib]
- [x] [serial] V1 [evidence=cairn/changes/first-stage-musl-rustc-driver-rlib/evidence/rustc-driver-frontier-2026-06-18.md] Record the real rerun `rustc_driver` failure and scratch continuation proving the rlib patch moves the frontier to dynamic proc-macro loading. r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_rustc_driver_rlib]
- [x] [serial] V2 [evidence=cairn/changes/first-stage-musl-rustc-driver-rlib/evidence/focused-validation-2026-06-18.md] Run focused script-generation tests, formatting, diff checks, and Cairn validation/gates. r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_rustc_driver_rlib]
