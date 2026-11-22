# Checks for `add-realize-build-cores`

| Record | What it shows |
|---|---|
| `checks-battery-1.log` | First run of the touched checks on the uncommitted change. It found a constant assertion (Clippy), a crunch-build unit test that expected `NIX_BUILD_CORES=1` at the default, a route CLI test whose default run reused the root built by the four-core run in the same store, and Cairn findings: scenarios without GIVEN, and task references on continuation lines. All were fixed before commit `b509335bc`. |
| `checks-battery-2.log` | The touched checks after those fixes: `cargo fmt --check` 0, Clippy with `-D warnings` 0, operator command contract PASS (215 commands), `cache_inputs_build_route` 18/18, `foreign_import_cli` 18 passed and 1 ignored, `operator_diagnostics` 30/30, `foreign_` bin unit tests 120 passed and 1 ignored, `cairn validate` 0 issues, all six gates PASS, `git diff --check` 0. |
| `checked-build-8ae03535b.log` | `nix build path:<clean export>#default` of the landed commit `8ae03535b`: exit 0, output `/nix/store/0j2489gc0g3nihxfix9864xrki53g19h-mantle-0.1.0`. Its check phase runs `cargo test --release --locked` for the root `mantle` package: 7,194 tests passed, 0 failed, 212 ignored across 88 test binaries. The first attempt was killed when another agent restarted the Nix daemon; this log is the rerun. |

The `crunch-build` lib run in `checks-battery-2.log` (debug profile) reports two failures. Neither is caused by this change:

- `worker::tests::watch_preflight_transitive_goal_limit_keeps_the_admitted_root` fails the same way on `main`. Its fixture registers `MAX_GOALS` derivations, which trips the registry's `debug_assert` on `MAX_ENTRIES`. This change does not touch `registry.rs`.
- `finish_gates::tests::real_glibc_audit_distinguishes_missing_and_opened_sonames` passes when run alone and fails only while the host is heavily loaded.

The checked build tests only the root `mantle` package, so it does not run `crunch-build`'s lib tests. In battery 2 that suite ran 818 tests: 816 passed and these two failed. This change's own `crunch-build` test, `build_request_applies_the_requested_core_count_to_native_and_foreign_profiles`, was among those that passed.
