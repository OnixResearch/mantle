# Tasks

- [x] [serial] I1 Bind source-root musl GCC unwind archives into native closure materialization as digest-bound runtime members. r[rust_package_planning.source_built_toolchain_closure.provider_fixed_point]
- [x] [serial] I2 Run explicit-closure rustc compatibility probes under the receipt-bound PATH and fail closed instead of generating an out-of-closure compatibility wrapper. r[rust_package_planning.source_built_toolchain_closure.provider_fixed_point]
- [x] [serial] I3 Generate receipt-bound C compiler aliases that expose the declared unwind archive as `libunwind.a` while keeping ambient PATH entries unavailable. r[rust_package_planning.source_built_toolchain_closure.provider_fixed_point]
- [x] [serial] V1 Add positive and negative focused tests for unwind alias materialization, receipt-bound compatibility probe PATH, and explicit-closure probe failure. r[rust_package_planning.source_built_toolchain_closure.provider_fixed_point]
  - Evidence: `evidence/final-validation-2026-06-25.md` records focused test results for `cargo_free_self_build::`, `native_toolchain_closure::`, and `source_toolchain_closure::`.
- [x] [serial] V2 Materialize the rerun38 provider/native closure manifest and record the provider-backed Cargo-free one-shot or fixed-point frontier. r[rust_package_planning.source_built_toolchain_closure.provider_fixed_point]
  - Evidence: `evidence/provider-fixed-point-proof-2026-06-25.md` records the enforced closure digest, successful provider-backed one-shot, and successful stage1/stage2 fixed point.
- [x] [serial] V3 Run formatting, focused Rust tests, diff checks, and Cairn validation/gates before claiming completion. r[rust_package_planning.source_built_toolchain_closure.provider_fixed_point]
  - Evidence: `evidence/final-validation-2026-06-25.md` records rustfmt, build, diff check, Cairn validate, and proposal/design/tasks gates.
