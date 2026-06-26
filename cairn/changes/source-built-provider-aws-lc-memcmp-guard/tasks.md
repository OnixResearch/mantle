## Evidence and investigation

- [x] [serial] I1 Record the current provider-backed fixed-point blocker with the exact AWS-LC memcmp guard diagnostic and receipt path. r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]
  - Evidence: `evidence/baseline-aws-lc-memcmp-guard-2026-06-25.md` records the stage1 receipt path and AWS-LC `COMPILER BUG DETECTED` / GCC PR95189 memcmp diagnostic.
- [x] [serial] I2 Inventory source-built native closure compiler members and determine whether any receipt-bound compiler can pass AWS-LC's guard without ambient host fallback. r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]
  - Evidence: `evidence/baseline-aws-lc-memcmp-guard-2026-06-25.md` records closure compiler members `cc` and `x86_64-linux-musl-gcc` and no receipt-bound `clang` route in the inspected manifest.

## Implementation

- [ ] [serial] I3 Implement deterministic compiler-guard handling: route to a proven receipt-bound safe compiler when available, otherwise emit a stable unsupported-compiler blocker. r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]
- [ ] [serial] I4 Keep compiler selection/policy as pure core over package, unit, target, and closure facts; keep process execution in the shell. r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]

## Verification

- [ ] [serial] V1 Add positive tests for receipt-bound safe compiler selection or stable blocker emission. r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]
- [ ] [serial] V2 Add negative tests proving Mantle does not bypass the guard via `HOST`/`TARGET` spoofing, ambient `CC`, arbitrary `CFLAGS`, or undeclared host compiler fallback. r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]
- [ ] [serial] V3 Rerun the real provider-backed fixed-point proof from current code and record success or the next deterministic blocker. r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]
- [ ] [serial] V4 Run formatting, focused tests, `git diff --check`, Cairn validation, and proposal/design/tasks gates before archive. r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard]
