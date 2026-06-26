## Evidence and baseline

- [x] [serial] I1 Record the current provider-backed native static-PIE CRT blocker. r[rust_package_planning.source_built_toolchain_closure.native_static_pie_crt]
  - Evidence: `evidence/baseline-native-static-pie-crt-2026-06-26.md` records the blocked proof, failed unit, guarded linker alias, and source-root musl PIE relocation diagnostic.

## Implementation

- [x] [serial] I2 Materialize the manifest-declared non-PIE CRT object into the receipt-bound C compiler alias runtime. r[rust_package_planning.source_built_toolchain_closure.native_static_pie_crt]
  - Evidence: `evidence/implementation-validation-2026-06-26.md` records the generated alias runtime checks and focused positive/negative tests.
- [x] [serial] I3 Map `rcrt1.o` arguments to the private runtime `crt1.o` while preserving the existing `-static-pie` to `-static` downgrade in direct and readable response-file argv. r[rust_package_planning.source_built_toolchain_closure.native_static_pie_crt]
  - Evidence: `evidence/implementation-validation-2026-06-26.md` records direct-argv and response-file alias tests.
- [x] [serial] I4 Fail closed when the explicit closure manifest lacks exactly one usable CRT member for alias normalization. r[rust_package_planning.source_built_toolchain_closure.native_static_pie_crt]
  - Evidence: `evidence/implementation-validation-2026-06-26.md` records missing-target-CRT and duplicate-target-CRT negative tests.

## Verification

- [x] [serial] V1 Run focused positive and negative alias tests plus an unset-env Mantle binary check. r[rust_package_planning.source_built_toolchain_closure.native_static_pie_crt]
  - Evidence: `evidence/implementation-validation-2026-06-26.md` records `cargo test -p mantle --bin mantle receipt_bound_c_compiler_alias -- --nocapture` and `env -u SNIX_BUILD_SANDBOX_SHELL cargo check -p mantle --bin mantle`.
- [x] [serial] V2 Rerun the real provider-backed fixed-point proof from current code and record success or the next deterministic blocker. r[rust_package_planning.source_built_toolchain_closure.native_static_pie_crt]
  - Evidence: `evidence/provider-fixed-point-success-2026-06-26.md` records pueue task 162 and bundle `mantle-source-built-rust-provider-fixed-point-native-static-pie-crt-2026-06-26-rerun4` with `fixed_point: true`.
- [x] [serial] V3 Run formatting, `git diff --check`, Cairn validation, and proposal/design/tasks gates before archive. r[rust_package_planning.source_built_toolchain_closure.native_static_pie_crt]
  - Evidence: `evidence/implementation-validation-2026-06-26.md` records formatting; `evidence/final-validation-2026-06-26.md` records `git diff --check`, Cairn validation, and proposal/design/tasks gates.
