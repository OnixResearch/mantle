## Evidence and investigation

- [x] [serial] I1 Record the current provider-backed fixed-point blocker with the exact AWS-LC memcmp guard diagnostic and receipt path. r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]
  - Evidence: `evidence/baseline-aws-lc-memcmp-guard-2026-06-25.md` records the stage1 receipt path and AWS-LC `COMPILER BUG DETECTED` / GCC PR95189 memcmp diagnostic.
- [x] [serial] I2 Inventory source-built native closure compiler members and determine whether any receipt-bound compiler can pass AWS-LC's guard without ambient host fallback. r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]
  - Evidence: `evidence/baseline-aws-lc-memcmp-guard-2026-06-25.md` records closure compiler members `cc` and `x86_64-linux-musl-gcc` and no receipt-bound `clang` route in the inspected manifest.

## Implementation

- [x] [serial] I3 Implement deterministic compiler-guard handling: route to a proven receipt-bound safe compiler when available, otherwise emit a stable unsupported-compiler blocker. r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]
  - Evidence: `evidence/implementation-validation-2026-06-25.md` records receipt-bound route selection, selected `cc` aliasing, and stable `unsupported-compiler-guard` mapping for AWS-LC's GCC PR95189 diagnostic.
- [x] [serial] I4 Keep compiler selection/policy as pure core over package, unit, target, and closure facts; keep process execution in the shell. r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]
  - Evidence: `evidence/implementation-validation-2026-06-25.md` records `src/source_toolchain_closure.rs` as the pure route-selection/validation core and `src/cargo_free_self_build.rs` / `src/rust_plan.rs` as the execution shells.

## Verification

- [x] [serial] V1 Add positive tests for receipt-bound safe compiler selection or stable blocker emission. r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]
  - Evidence: `evidence/implementation-validation-2026-06-25.md` records passing `cargo test -p mantle --bin mantle c_compiler_route`, `compiler_guard`, `selected_compiler`, and `receipt_bound_path_aliases` focused positive coverage.
- [x] [serial] V2 Add negative tests proving Mantle does not bypass the guard via `HOST`/`TARGET` spoofing, ambient `CC`, arbitrary `CFLAGS`, or undeclared host compiler fallback. r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]
  - Evidence: `evidence/implementation-validation-2026-06-25.md` records negative coverage for ambiguous non-Clang routes, invalid route JSON, ambient `CC`/`CFLAGS`, and `HOST`/`TARGET` spoofing.
- [x] [serial] V3 Rerun the real provider-backed fixed-point proof from current code and record success or the next deterministic blocker. r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]
  - Evidence: `evidence/provider-rerun-stage-receipt-2026-06-25.md` records pueue task 22. AWS-LC `aws-lc-sys@0.39.1` build-script metadata succeeded with a selected receipt-bound `cc` route recorded in the durable stage receipt; the current blocker moved to `snix-build` compile-time `SNIX_BUILD_SANDBOX_SHELL` env handling.
- [x] [serial] V4 Run formatting, focused tests, `git diff --check`, Cairn validation, and proposal/design/tasks gates before archive. r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]
  - Evidence: `evidence/final-validation-2026-06-26.md` records rustfmt check, `git diff --check`, full `cargo test -p mantle --bin mantle`, Cairn validation, and proposal/design/tasks gates passing.
