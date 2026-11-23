# Checks: surface foreign realization diagnostics

Environment for every command (from the follow-up spec):

```
export PATH=/nix/store/mvj6y8k3xs35cj7kq937xi44q4hl3r93-bubblewrap-0.12.0/bin:/nix/store/2msw5nclhpsm80z65dqmjiv8kkras989-clang-wrapper-21.1.8/bin:$PATH
export RUSTC_WRAPPER= SNIX_BUILD_SANDBOX_SHELL=/bin/sh CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR=/datapool/mantle-store/followups-20261122/mantle-target
export TMPDIR=/datapool/mantle-store/followups-20261122/mantle-tmp
```

## Cargo

| Command | Result |
|---|---|
| `cargo test -p crunch-store --lib` | `test result: FAILED. 471 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 200.74s` — the single failure was `query::tests::store_verify_read_failure_does_not_persist_pathinfo`, an environment `SUN_LEN` failure under the long spec `TMPDIR`; rerunning it alone with `TMPDIR=/tmp/mf-tmp` gives `test result: ok. 1 passed`. |
| `cargo test -p mantle --test cache_inputs_build_route` | `test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.41s` |
| `cargo test -p mantle --test foreign_import_cli` | `test result: ok. 18 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 4.20s` |
| `cargo test -p mantle --bin mantle -- foreign_` | `test result: ok. 122 passed; 0 failed; 1 ignored; 0 measured; 2907 filtered out; finished in 97.47s` |
| `cargo test -p crunch-store --lib -- pull::tests::http_closure_content_admission handle::tests::cached_path_info pull::tests::http_nar_refusal_does_not_echo_cache_credentials` | `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 468 filtered out; finished in 0.33s` |
| `cargo test -p mantle --test cache_inputs_build_route -- cache_inputs_build_or_reuse_route_keeps_the_built_root_log` | `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 3.67s` |
| `cargo fmt --check -p mantle -p crunch-store -p crunch-build -p crunch-pipeline` | clean: no diff lines |
| `cargo clippy -p mantle -p crunch-store -p crunch-build -p crunch-pipeline --all-targets --no-deps -- -D warnings` | `Finished `dev` profile [unoptimized + debuginfo] target(s) in 4m 35s` with no warning or error lines. |
| `git diff --check` | exit 0, no output |

## Cairn

| Command | Result |
|---|---|
| `cairn validate --root .` | `"issues": []`, `"valid": true` |
| `cairn gate proposal surface-foreign-realization-diagnostics --root .` | `"verdict": "PASS"` |
| `cairn gate design surface-foreign-realization-diagnostics --root .` | `"verdict": "PASS"` |
| `cairn gate tasks surface-foreign-realization-diagnostics --root .` | `"verdict": "PASS"` |

## Old-code demonstration

Production behavior was temporarily restored to the pre-change shape (the same binary, one edit at
a time), the new regression was run once, and the edit was reverted.

| Behavior | Old-code run | Restored run |
|---|---|---|
| Content-admission refusals name their cause | `cargo test -p crunch-store --lib -- pull::tests::http_closure_content_admission` → `test result: FAILED. 0 passed; 2 failed; 0 ignored; 0 measured; 470 filtered out`, with `message: store: http-closure-content-admission-failed: cin6js…-status-refusal-child` (no class/detail) | `test result: ok. 2 passed` in the final run |
| Successful realizations keep build logs | `cargo test -p mantle --test cache_inputs_build_route -- cache_inputs_build_or_reuse_route_keeps_the_built_root_log` → `test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 18 filtered out; finished in 5.15s`, panicking at `realization must keep the built root log at …/state/logs/k0z6rxfr9xvjpfympjdv9958441cb44w-parent.drv.log` | `cache_inputs_build_route`: 19 passed |

The refusal classes and detail are therefore pinned by tests that fail without the change, and the
log regression is pinned by the CLI test that fails when `write_realization_logs` is bypassed.

## Follow-up: log failures under `--json` (orchestrator review)

Review found that a log-write failure was printed only in human mode, so under `--json` (the mode
the pilot runs use) it was invisible. `run_realize` now writes each failure as one
`{"diagnostic_persistence_failure": {...}}` JSON line on stderr under `--json`, and the design,
spec, and task T2.2 say so. New CLI test
`cache_inputs_build_or_reuse_route_reports_an_unwritable_log_dir_as_json_and_still_succeeds` blocks
`<state-dir>/logs` with a regular file: realize exits 0, the receipt reports the root `built`, and
stderr holds exactly one `create-log-dir` failure line and no `WARNING:` prose.

| Command | Result |
|---|---|
| `cargo test -p mantle --test cache_inputs_build_route` | `test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.53s` |
| `cargo fmt --check -p mantle` | exit 0, no diff |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0, no warning or error lines |
| `cairn validate --root .` (pinned 007ba519) | `{"valid":true,"issues":0}` |
| `cairn gate proposal\|design\|tasks surface-foreign-realization-diagnostics` | PASS, PASS, PASS |
| `git diff --check` | exit 0 |

## Follow-up: failed root logs on realize (live Octet finding)

A live Octet run showed a builder-stage failure producing no `<state-dir>/logs/<drv>.log`
(`mantle log --list` said "No build logs found") because `write_realization_logs` only read
`FailedGoal.build_log`. `mantle build`'s failure-log gate, drv key, `origin …` detail, and writer are
now extracted as `build_cmd::write_failed_goal_log` and shared; `mantle build`'s output is unchanged
by construction. No CLI test drives a failing realize (the existing realize CLI tests succeed or are
rejected before building), so the new coverage is the `src/foreign_realization_shell.rs` unit tests
listed below.

| Command | Result |
|---|---|
| `cargo test -p mantle --bin mantle -- foreign_realization_shell::tests --test-threads=1` | `test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 3026 filtered out; finished in 0.01s` |
| `cargo test -p mantle --bin mantle -- foreign_ --test-threads=1` | `test result: ok. 124 passed; 0 failed; 1 ignored; 0 measured; 2907 filtered out; finished in 132.21s` |
| `cargo test -p mantle --bin mantle -- build_cmd` | `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 3019 filtered out; finished in 0.02s` |
| `cargo test -p mantle --test cache_inputs_build_route` | `test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.16s` |
| `cargo test -p mantle --test foreign_import_cli` | `test result: ok. 18 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 14.00s` |
| `cargo test -p mantle --test operator_diagnostics -- saved_log_path failure_log --test-threads=1` | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 27 filtered out; finished in 4.04s` |
| `cargo test -p mantle --test operator_diagnostics -- build_human_failure_summary_matches_json_facts build_json_success_reports_log_persistence_failure --test-threads=1` | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 28 filtered out; finished in 4.81s` |
| `cargo fmt --check -p mantle` | exit 0, no diff |
| `cargo clippy -p mantle --all-targets --no-deps -- -D warnings` | exit 0, `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 3m 06s`, no warning or error lines |
| `cairn validate --root .` | `"issues": []`, `"valid": true` |
| `cairn gate proposal\|design\|tasks surface-foreign-realization-diagnostics --root .` | `"verdict": "PASS"` for all three |
| `git diff --check` | exit 0, no output |

## Checked build and live scope

Clean `git archive` export of `5c58ea50b3e23c78b5a14cdf2db93b171e05a6e1`:
`nix build -L --print-out-paths path:<export>#default` with local builders exited 0
(7,205 passed, 0 failed, 212 ignored across 89 suite results). Output package:
`/nix/store/r5z7wyxna58ig4gkdbiwdyqxk1z3zaqf-mantle-0.1.0`.
Binary BLAKE3 `a393733725dce076950fe938cc765e3e640dbf242f1ef9236d5a8b0e7ef8b421`;
retained build log BLAKE3 `21a0b8a2d00856859128a05597e400468d484f817dc6eed96d66c142937841ee`.

The real Octet rerun used the preceding checked Mantle `5f6a57b46`, not
`5c58ea50b`. `mantle log --list` identified the successfully built root, and
`mantle log <root-drv>` returned `pid 2's new affinity list: 0-7`.
The remote sandbox hit the 45-minute Lean deadline; local fallback succeeded.
The final receipt was successful. This rerun verifies the successful-root
log, not the subsequent failed-root logging change or the cache refusal classes.
Those paths are covered by the targeted tests above and the checked build.

The operator's retained `followups/live-rerun-2/README.md` (BLAKE3
`4214481891441c0545701b79a2dcd62b772f234ae85013a087ad6d845b5ae297`)
indexes the exact inputs, logs, and receipts. On Octet `b6137b5b`, the
Mantle and Nix roots were byte-identical, Octet's decisions were identical,
and local reuse took 226 seconds with no builds or downloads. Cairn still
reported both cases `stale` against the prior-revision fixture registry
(`convergence.registry_unverified`, `convergence.revision_mismatch`); this
run does not claim current assurance.
