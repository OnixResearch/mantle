## Evidence and baseline

- [x] [serial] I1 Record the current unset-env compile blocker for vendored `snix-build`. r[rust_package_planning.source_built_toolchain_closure.snix_sandbox_shell_compile_env]
  - Evidence: `evidence/baseline-snix-sandbox-shell-compile-env-2026-06-26.md` records pueue task 18 failing with `environment variable `SNIX_BUILD_SANDBOX_SHELL` not defined at compile time` in `vendor/snix-build/src/buildservice/{oci,bwrap}.rs`.

## Implementation

- [x] [serial] I2 Replace mandatory compile-time `SNIX_BUILD_SANDBOX_SHELL` reads with an optional placeholder default without adding source-built sandbox-shell claims. r[rust_package_planning.source_built_toolchain_closure.snix_sandbox_shell_compile_env]
  - Evidence: `evidence/implementation-validation-2026-06-26.md` records `option_env!` placeholder handling in vendored `snix-build` and Mantle diagnostics.
- [x] [serial] I3 Keep runtime shell selection separate from compile-time availability. r[rust_package_planning.source_built_toolchain_closure.snix_sandbox_shell_compile_env]
  - Evidence: `evidence/implementation-validation-2026-06-26.md` records that bwrap runtime selection still prefers explicit runtime `SNIX_BUILD_SANDBOX_SHELL` and treats `/bin/sh` as a placeholder.

## Verification

- [x] [serial] V1 Prove vendored `snix-build` compiles with `SNIX_BUILD_SANDBOX_SHELL` unset. r[rust_package_planning.source_built_toolchain_closure.snix_sandbox_shell_compile_env]
  - Evidence: `evidence/implementation-validation-2026-06-26.md` records pueue task 25 passing `cargo check -p snix-build --lib` with the env unset.
- [x] [serial] V2 Prove the Mantle binary compile path no longer depends on ambient `SNIX_BUILD_SANDBOX_SHELL`. r[rust_package_planning.source_built_toolchain_closure.snix_sandbox_shell_compile_env]
  - Evidence: `evidence/implementation-validation-2026-06-26.md` records pueue task 35 passing `cargo check -p mantle --bin mantle` with the env unset.
- [x] [serial] V3 Rerun the real provider-backed fixed-point proof from current code and record success or the next deterministic blocker. r[rust_package_planning.source_built_toolchain_closure.snix_sandbox_shell_compile_env]
  - Evidence: `evidence/provider-rerun-next-blocker-2026-06-26.md` records the provider-backed proof with `SNIX_BUILD_SANDBOX_SHELL` unset advancing to 660 units, passing the prior `snix-build` frontier, and blocking next at the native Mantle binary `rustc-failed` diagnostics.
- [x] [serial] V4 Run formatting, focused tests, `git diff --check`, Cairn validation, and proposal/design/tasks gates before archive. r[rust_package_planning.source_built_toolchain_closure.snix_sandbox_shell_compile_env]
  - Evidence: `evidence/final-validation-2026-06-26.md` records unset-env `snix-build` tests/check, unset-env Mantle binary check, `cargo fmt --check -p mantle -p snix-build`, `git diff --check`, `cairn validate`, and proposal/design/tasks gates all exiting 0.
