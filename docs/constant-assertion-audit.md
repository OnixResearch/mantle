# Constant-Assertion Audit (`#60`)

## Summary

This audit covers constant-only assertions (`assert!`, `assert_eq!`, `assert_ne!`, `debug_assert!`, `debug_assert_eq!`, `debug_assert_ne!`) appearing inside production function bodies (outside `#[cfg(test)]` modules, `const _: ...` compile-time checks, inline `const { ... }` blocks, and test/example/bench targets).

## Verification & Enforcement Tooling

- Guard script: `scripts/check-constant-assertions.py`
  - Default check (`python3 scripts/check-constant-assertions.py`): enforces zero inside-`fn` constant-only assertions across M4-C.1 & M4-C.2 owned/clean targets (`src/shell_cmd.rs`, `src/lib.rs`, `src/main.rs`, `src/bin/crunch.rs`, `src/cli.rs`, `src/rust_plan_execution.rs`, `src/build_correctness.rs`, `src/profile_cli.rs`, `src/machine_contract_producer_tests.rs`, `crates/mantle-rust-plan-core/src`, `crates/mantle-rust-plan-app/src`, `crates/crunch-test-core/src`, `crates/crunch-test-app/src`, `crates/crunch-test/src`, `src/rust_plan.rs`, `src/stagex_transition.rs`). Currently: **27 owned/clean files, 0 violations**.
  - Self-test (`python3 scripts/check-constant-assertions.py --self-test`): validates **10 positive** cases (top-level `const _: () = { ... };`, top-level `const _: () = assert!(...);`, inline `const { ... }`, string-literal macro false-positive rejection, runtime `&str` check, runtime `&[u8]` check against a constant bound, `static` `OnceLock` check, digit-containing runtime identifier `v1_name`, mixed runtime/const `u256_count <= U256_MAX && V1_NAME.len() <= u256_count`, and `#[cfg(test)]` module) and **11 negative** cases (`debug_assert!(!SIDECAR_FILENAME.is_empty())`, `assert!(MAX_ITEMS > 0, "must be positive")`, `debug_assert_ne!(MODE_A, MODE_B)`, `assert!("abc".len() == 3)`, `assert_ne!(1_usize, 2_usize)`, `debug_assert!(0x10_u64 > 0)`, `assert_eq!(0b1010_u8, 10_u8)`, `debug_assert!(0o77_u32 == 63)`, `assert!(1_000_i32 == 1_000_i32)`, `debug_assert!(1_000.0_f64 > 0.0_f64)`, and `assert!(!V1_NAME.is_empty() && U256_MAX > 0)` inside a production function).
  - Full-repository inventory (`python3 scripts/check-constant-assertions.py --audit-all`): scans all production Rust files under `src/` and `crates/*/src/` while attributing M4-C.1, M4-C.2, Forbidden Lane C, and Other Unowned files.

## Fixes Applied in M4-C.1

1. **`src/shell_cmd.rs` (`SIDECAR_FILENAME` compile-time checks, runtime invariants & input guards)**:
   - Replaced all **8** repeated inside-`fn` `SIDECAR_FILENAME` assertions from `main:src/shell_cmd.rs` (`df4a2c2c`: `cmd_shell`, `build_shell_target`, `exec_hook`, `exec_plan` — 2 each) with a single top-level compile-time block:
     ```rust
     const _: () = {
         assert!(!SIDECAR_FILENAME.is_empty());
         assert!(SIDECAR_FILENAME.as_bytes()[0] == b'.');
         assert!(!SHELL_LEASE_ID_DOMAIN.is_empty());
     };
     ```
   - Replaced the repeated constant assertions with non-compound runtime `debug_assert!` / `debug_assert_eq!` invariants on real runtime state across `cmd_shell`, `select_executable_hook`, `load_shell_sidecar`, `resolve_shell_exec_mode`, `shell_project_target`, `build_shell_target`, `shell_root_registration`, `plan_shell_root_registration`, `shell_lease_identity`, `hash_shell_lease_field`, `resolve_host_shell`, `snapshot_host_env`, `resolve_hook_shell`, `exec_hook`, `validate_with_paths`, `resolve_in_path`, `prepare_exec_command`, and `exec_plan`.
   - Extracted `resolve_host_shell` (filtering empty `SHELL=""` to `/bin/sh` fallback) and `select_executable_hook` (skipping disabled, missing, or empty hooks before `exec_hook`), plus an explicit empty `command_argv[0]` guard at the top of `cmd_shell`, verified by positive unit tests (`resolve_host_shell_falls_back_to_bin_sh_when_missing_or_empty`, `select_executable_hook_skips_disabled_missing_and_empty_hooks`) and 7 `#[cfg(debug_assertions)] #[should_panic]` negative invariant tests in `src/shell_cmd.rs::tests`.

2. **`src/shell_cmd.rs` (`hash_shell_lease_field` & `shell_lease_identity`)**:
   - Implemented length-prefixed BLAKE3 domain separation using `u128::try_from(bytes.len())` (`byte_count`), checked via `debug_assert_eq!(usize::try_from(byte_count).ok(), Some(bytes.len()))` before feeding the **big-endian** 16-byte length prefix (`&byte_count.to_be_bytes()`) and `bytes` payload into the hasher (with deterministic `b"len-overflow"` fallback on `Err(_)`).
   - Added `shell_lease_identity_distinguishes_field_boundaries_and_is_deterministic` in `src/shell_cmd.rs::tests`, verifying deterministic self-consistency and that distinct field boundaries (`("ab", "c")` vs `("a", "bc")` with `now_unix_s: TEST_NOW_UNIX_S`, plus `TEST_NOW_UNIX_S + 1`) produce distinct `"b3:"` digests.

3. **Clean owned targets & `crates/crunch-test/src/lib.rs` `HARDENING-BACKLOG` note**:
   - Confirmed **0** inside-`fn` constant-only assertions across `src/cli.rs`, `src/lib.rs`, `src/main.rs`, `src/bin/crunch.rs`, `src/profile_cli.rs`, `src/build_correctness.rs`, `src/rust_plan_execution.rs`, `src/machine_contract_producer_tests.rs`, `crates/mantle-rust-plan-{core,app}/src`, and `crates/crunch-test-{core,app}/src` / `crates/crunch-test/src`.
   - `crates/crunch-test/src/lib.rs` carries a pre-existing `#![allow(tigerstyle::ambient_clock, tigerstyle::contradictory_time)]` (`// HARDENING-BACKLOG 2026-10-22`, scoped to spawned-process deadline enforcement) and contains **0** constant-only inside-`fn` assertions.

## CLI Consolidation Context (`#58` + `#78`): `src/lib.rs` Backlog & Build Timings

- **`src/lib.rs` consolidation & backlog**:
  - `src/lib.rs` declares `225` modules (`224` unique modules from `main:src/lib.rs` ∪ `main:src/main.rs` at `df4a2c2c`, plus `mod cli;`), while `src/main.rs` and `src/bin/crunch.rs` are 3-line shims invoking `mantle::cli_main()`.
  - `155` legacy modules consolidated from `main:src/main.rs` (`df4a2c2c`) carry `// HARDENING-BACKLOG 2026-11-25` outer `#[allow(tigerstyle::...)]` attributes in `src/lib.rs` scoped strictly to their pre-existing lints (including `source_built_fixed_point_planning` and `source_built_fixed_point_receipt`, which belong to forbidden `src/source_built_fixed_point*.rs` files); `34` `dead_code` suppression attributes remain (`33` `#[allow(dead_code...)]` plus `1` `#[cfg_attr(not(target_os = "linux"), allow(dead_code))]`, down from `35` in M4-C.1 `3f801e52` after M4-C.2 retired `#[allow(dead_code)]` on `mod stagex_transition;` — `28` retired relative to `main:src/main.rs`'s `62`, `30` retired overall across `main:src/{lib,main}.rs`, `0` added).
  - **0** `#[allow(tigerstyle::...)]` suppressions appear in M4-C.1 refactored/owned files (`src/cli.rs`, `src/shell_cmd.rs`, `src/profile_cli.rs`, `src/build_correctness.rs`, `src/main.rs`, `src/bin/crunch.rs`, or clean pre-existing `src/lib.rs` modules).
- **Build timings on `aspen1` (`CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_INCREMENTAL=0`)**:
  - Baseline (`main` `df4a2c2c`, duplicate `src/main.rs` binary target via `cargo check -p mantle --bins`): `185.71s` clean (`user 316.55s`, `sys 63.35s`); `134.53s` incremental after comment edit in `src/main.rs` (`user 169.90s`, `sys 9.18s`).
  - Post-consolidation (`cargo check -p mantle --lib --bins`): `1m43.927s` clean (`CARGO_TARGET_DIR=/dev/shm/kiln-swarm/timing-c`, `user 2m51.612s`, `sys 0m51.608s`); `3.059s` incremental no-op; `1.363s` incremental after `touch src/main.rs`; `13.911s` incremental after `touch src/cli.rs`.

## Fixes Applied in M4-C.2

1. **`src/stagex_transition.rs` (`L6119` `BASH_COUNT_BOUNDS` assertion) & `src/lib.rs`**:
   - Replaced the inside-`fn` constant-only `assert!(BASH_COUNT_BOUNDS[0] <= BASH_COUNT_BOUNDS[1]);` in `require_binutils_non_coreutils_counts` (`3f801e52:src/stagex_transition.rs:6119`) with module-level `BINUTILS_BASH_COUNT_BOUNDS` (`[7_619, 7_627]`) + top-level `const _: ()` compile-time checks (`BINUTILS_BASH_COUNT_BOUNDS[0] > 0` and `BINUTILS_BASH_COUNT_BOUNDS[0] <= BINUTILS_BASH_COUNT_BOUNDS[1]`, alongside `BINUTILS_MKDIR_EXEC_COUNT_BOUNDS` checks) and runtime invariants `debug_assert!(paths.bash.is_absolute());` and `debug_assert!(!counts.is_empty());` (alongside the pre-existing `assert_eq!(sed_count, report.sed_invocation_count + SED_COUNT_OFFSET);`).
   - Moved `EXPECTED_PROMOTION_COUNT` into `src/stagex_transition.rs::tests` and removed `#[allow(dead_code)]` on `mod stagex_transition` in `src/lib.rs`.
   - Promoted `src/stagex_transition.rs` into `OWNED_CLEAN_TARGETS` in `scripts/check-constant-assertions.py` (**0 violations**).

2. **`src/rust_plan.rs` & `crates/mantle-rust-plan-core/src/plan.rs`**:
   - Eliminated all **32** inside-`fn` constant-only assertions from `3f801e52:src/rust_plan.rs`: replaced **30** in `src/rust_plan.rs` with top-level `const _: ()` compile-time checks plus runtime `debug_assert!`/`assert!` invariants on live values, and moved the **2** diagnostic-redaction constant checks (`REDACTED_DIAGNOSTIC_TEMP_PATH`, `TEMP_PATH_PREFIXES`) into top-level `const _: ()` in `crates/mantle-rust-plan-core/src/plan.rs` alongside runtime `debug_assert!` invariants in `redact_temp_paths` (**0** constant-only inside-`fn` assertions in both files).
   - Promoted `src/rust_plan.rs` into `OWNED_CLEAN_TARGETS` in `scripts/check-constant-assertions.py` (leaving `M4_C2_TARGETS` empty; **0 violations** across all **27** owned/clean files).

## Repository-Wide Backlog Snapshot (Owned vs. Unowned Breakdown)

Measured with `python3 scripts/check-constant-assertions.py --audit-all`:

| Bucket | Files | Constant-Only Assertions | Notes |
| --- | ---: | ---: | --- |
| M4-C.1 owned (`src/shell_cmd.rs`) | 0 | 0 | Enforced clean |
| M4-C.2 owned (`src/rust_plan.rs`, `src/stagex_transition.rs`) | 0 | 0 | Cleaned and promoted to `OWNED_CLEAN_TARGETS` |
| Clean owned targets (`OWNED_CLEAN_TARGETS`) | 0 | 0 | Enforced clean across 27 files |
| Forbidden for Lane C (owned by other lanes) | 12 | 24 | Untouched per worktree boundary |
| Other unowned workspace files (`src/`: 59/197, `crates/`: 23/65) | 82 | 262 | Future hardening backlog (`--strict` mode tracks full cleanup) |
| **Total repository production files** | **94** | **286** | Updated post-M4-C.2 baseline |

### Forbidden for Lane C (`12` files, `24` assertions)
- `crates/crunch-build/src/build_request.rs` (`1`): `L904`
- `crates/crunch-build/src/dynamic_plan.rs` (`4`): `L890, L891, L966, L967`
- `crates/crunch-build/src/environment_policy.rs` (`1`): `L600`
- `crates/crunch-build/src/workspace.rs` (`2`): `L528, L529`
- `crates/crunch-build/src/workspace_shell.rs` (`1`): `L679`
- `crates/crunch-store/src/pull.rs` (`1`): `L1011`
- `src/elf_local_symbol_core.rs` (`1`): `L185`
- `src/remote_build/attenuated_admission.rs` (`3`): `L204, L335, L336`
- `src/remote_build.rs` (`2`): `L12927, L12928`
- `src/self_build.rs` (`6`): `L2133, L2466, L2467, L2544, L3034, L3168`
- `src/source_built_fixed_point_resume_bundle.rs` (`1`): `L204`
- `src/source_built_fixed_point_shell.rs` (`1`): `L1673`

### Other Unowned Summary (`82` files, `262` assertions)
- **`src/` (`59` files, `197` assertions)** — top files (`>= 5` assertions): `src/cargo_free_self_build.rs` (`14`), `src/stagex_binutils.rs` (`14`), `src/foreign_import_cmd.rs` (`13`), `src/bootstrap_validate.rs` (`10`), `src/project_cmd.rs` (`10`), `src/ast_grep_evidence.rs` (`9`), `src/frontend_artifact_export.rs` (`8`), `src/full_source_provider.rs` (`8`), `src/cairn_release_handoff.rs` (`6`), `src/frontend_artifact_spec.rs` (`6`), `src/offline_cargo.rs` (`6`), `src/nickel_export_core_adapter.rs` (`5`), `src/nix_free_demo_cmd.rs` (`5`), `src/stagex_bash_full.rs` (`5`), plus `45` files (`1–4` assertions each; `78` assertions).
- **`crates/` (`23` files, `65` assertions)** — top files (`>= 3` assertions): `crates/crunch-release-core/src/determinism.rs` (`12`), `crates/crunch-release-core/src/manifest.rs` (`10`), `crates/crunch-release-core/src/function_address_binding.rs` (`8`), `crates/crunch-release-core/src/opaque_evidence.rs` (`5`), `crates/crunch-release-core/src/nix_witness.rs` (`3`), plus `18` files (`1–2` assertions each; `27` assertions).
