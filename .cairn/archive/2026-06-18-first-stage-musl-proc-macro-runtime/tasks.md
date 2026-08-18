# Tasks

- [x] [serial] I1 Add a musl-host-only first-stage proc-macro runtime setup that copies source-root musl `libc.so`, patches `run_rustc` `LD_LIBRARY_PATH`, scrubs ambient Cargo wrappers, and fails closed on unexpected source/runtime shape. r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_proc_macro_runtime]
- [x] [serial] V1 [evidence=cairn/changes/first-stage-musl-proc-macro-runtime/evidence/proc-macro-runtime-frontier-2026-06-18.md] Record the real `tracing_attributes` blocker and scratch continuation proving the runtime path moves the frontier. r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_proc_macro_runtime]
- [x] [serial] V2 [evidence=cairn/changes/first-stage-musl-proc-macro-runtime/evidence/focused-validation-2026-06-18.md] Run focused script-generation tests, formatting, diff checks, and Cairn validation/gates. r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_proc_macro_runtime]
