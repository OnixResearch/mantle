## Evidence and baseline

- [x] [serial] I1 Record the current provider-backed Mantle binary warning blocker. r[rust_package_planning.source_built_toolchain_closure.mantle_bin_warning_frontier]
  - Evidence: `evidence/baseline-mantle-bin-warning-frontier-2026-06-26.md` records the blocked provider proof with 660 executed units and a failed native Mantle binary unit whose diagnostics name unused test-only items.

## Implementation

- [x] [serial] I2 Scope the `cargo_import` `BTreeSet` import to tests only. r[rust_package_planning.source_built_toolchain_closure.mantle_bin_warning_frontier]
  - Evidence: `evidence/implementation-validation-2026-06-26.md` records the focused `cargo_import` tests and unset-env Mantle binary check without the old `BTreeSet` warning.
- [x] [serial] I3 Scope the musl-target GCC alias helper constant to tests only. r[rust_package_planning.source_built_toolchain_closure.mantle_bin_warning_frontier]
  - Evidence: `evidence/implementation-validation-2026-06-26.md` records the focused toolchain alias tests and unset-env Mantle binary check without the old `MUSL_TARGET_GCC_ALIAS` warning.
- [x] [serial] I4 Add scoped dormant-code allowances for intentionally parked provider/front-end/native-planner surfaces. r[rust_package_planning.source_built_toolchain_closure.mantle_bin_warning_frontier]
  - Evidence: `evidence/implementation-validation-2026-06-26.md` records the unset-env Mantle binary check exiting 0 without rustc unused-item warnings after scoped allowances.

## Verification

- [x] [serial] V1 Run focused unit tests and unset-env Mantle binary checks showing the local warnings no longer appear. r[rust_package_planning.source_built_toolchain_closure.mantle_bin_warning_frontier]
  - Evidence: `evidence/implementation-validation-2026-06-26.md` records `cargo test -p mantle --bin mantle cargo_import_plan`, `cargo test -p mantle --bin mantle toolchain_path_aliases`, `cargo test -p mantle --bin mantle rust_bootstrap_patch_plan`, and `env -u SNIX_BUILD_SANDBOX_SHELL cargo check -p mantle --bin mantle` all exiting 0 with an assertion that no rustc unused-item warnings matched.
- [x] [serial] V2 Rerun the real provider-backed fixed-point proof from current code and record success or the next deterministic blocker. r[rust_package_planning.source_built_toolchain_closure.mantle_bin_warning_frontier]
  - Evidence: `evidence/provider-rerun-next-blocker-2026-06-26.md` records pueue task 136, `fixed_point: false`, `unit_count: 679`, and the next blocker as a musl static-PIE link failure rather than the old unused-item warning frontier.
- [x] [serial] V3 Run formatting, `git diff --check`, Cairn validation, and proposal/design/tasks gates before archive. r[rust_package_planning.source_built_toolchain_closure.mantle_bin_warning_frontier]
  - Evidence: `evidence/final-validation-2026-06-26.md` records `cargo fmt --check -p mantle -v`, `git diff --check`, `cairn validate`, and proposal/design/tasks gates all exiting 0.
