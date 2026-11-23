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
