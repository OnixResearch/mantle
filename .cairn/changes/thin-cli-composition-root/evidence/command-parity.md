# Command parity (V3)
Observed 2026-10-29 to 2026-10-30 (local run timestamps below). Focused test, smoke and skip-visibility observations for one selected source tree. This file records what ran and what it printed; it does not accept I6, V1, V2, V4 or V5, and it does not complete the change. See Non-claims.
## Subject
- Git commit `5fc2ad4b3dfb1ae80ff5060cdd8fb6670230e316`, tree `35d06fef314555b7df2e47d1f15294caa29a47ad`; working tree status after the parity run: empty (clean).
- Test convention: serialized (`-- --test-threads 1`), as in the repository's first-party gate.
- Exact UTF-8 stream text and BLAKE3 identities for every retained command are in `command-parity-captures-2026-10-30.json` beside this file. Quoted output in this Markdown file has trailing whitespace removed from each line; the captures file keeps the exact bytes.
- Runner: `run-commands.py` (BLAKE3 `93e431b88e990538ee266521a2ade108881d0b5d51f73b32f94ba461acd67b3d`), request BLAKE3 `01a937a8c612232434d78126172765516bc95f62c76906a0cf26a29b8a4c1516`.
- Retained run directory: `/home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/thin-cli-parity-v87`.
- Started 2026-10-29T23:09:50.352990-04:00, finished 2026-10-29T23:59:46.015680-04:00; driver exit 0; selected-subject digests unchanged: True.

## Test commands
| # | Label | Exit | Elapsed | stdout BLAKE3 (first 16) | stderr BLAKE3 (first 16) |
|---|---|---|---|---|---|
| 1 | `source-identity` | 0 | 0.0s | `05fd3aae0655d553` | `af1349b9f5f9a1a6` |
| 2 | `application-contract-tests` | 0 | 6.2s | `b2c0a3a8c7312a94` | `b33952e940f8b989` |
| 3 | `mantle-bin-tests` | 0 | 2239.3s | `d9ce6d49658b1184` | `90a052912cb71b64` |
| 4 | `user-profile-lib-tests` | 0 | 15.0s | `8fc541f636e37c12` | `5cbe4dd017992e47` |
| 5 | `rustc-wrapper-run-daemon-tests` | 0 | 195.8s | `c988e340e61aa956` | `25264a338348d3ed` |
| 6 | `family-parity-cli-tests` | 0 | 440.9s | `217f023c01f9f078` | `179f350c960e3fef` |
| 7 | `evaluation-and-example-tests` | 0 | 98.2s | `e024e72188e4003f` | `fb53479af86c1965` |
| 8 | `worktree-status-after` | 0 | 0.1s | `af1349b9f5f9a1a6` | `af1349b9f5f9a1a6` |

Exact argv:
- `source-identity`: `git rev-parse HEAD HEAD^{tree}`
- `application-contract-tests`: `cargo test --locked --offline -p mantle-application-contract -- --test-threads 1`
- `mantle-bin-tests`: `cargo test --locked --offline -p mantle --bin mantle -- --test-threads 1`
- `user-profile-lib-tests`: `cargo test --locked --offline -p mantle --lib user_profile_cmd::tests -- --test-threads 1`
- `rustc-wrapper-run-daemon-tests`: `cargo test --locked --offline -p crunch-rustc-wrapper run_daemon -- --test-threads 1`
- `family-parity-cli-tests`: `cargo test --locked --offline -p mantle --test operator_diagnostics --test rust_plan_cli --test cargo_free_self_build_cli --test remote_credentials_cli --test remote_stdio_cli --test project_cli --test project_refresh_cli --test project_build_smoke --test source_bundle_cli --test source_bundle_hydration_cli --test release_cli --test store_gc_cli --test store_archive_cli --test bootstrap_validate_cli --test bootstrap_parity_cli --test transcript_cli --test kernel_bundle_oci_cli --test kernel_bundle_oci_registry_cli -- --test-threads 1`
- `evaluation-and-example-tests`: `cargo test --locked --offline -p mantle --test evaluator_budget_cli --test causal_build_trace --test integration --test example_projects -- --test-threads 1`
- `worktree-status-after`: `git status --porcelain`

## Observed test results by target
Result lines are quoted verbatim. Where a test re-executes the test binary, its own libtest summary appears nested inside the parent block; those nested blocks are excluded from the tables and sums, and the count of excluded blocks is stated.

### `application-contract-tests` (exit 0)
| Target | Declared tests | Result line (verbatim) |
|---|---|---|
| `unittests src/lib.rs` | 126 | `test result: ok. 126 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` |
| `tests/contract_fixtures.rs` | 8 | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |
| `tests/port_fixtures.rs` | 8 | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |
| `doc-tests mantle_application_contract` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |

Sum over top-level result lines: 142 passed, 0 failed, 0 ignored, 0 filtered out.

### `mantle-bin-tests` (exit 0)
| Target | Declared tests | Result line (verbatim) |
|---|---|---|
| `unittests src/main.rs` | 2848 | `test result: ok. 2755 passed; 0 failed; 93 ignored; 0 measured; 0 filtered out; finished in 2110.60s` |

Sum over top-level result lines: 2755 passed, 0 failed, 93 ignored, 0 filtered out.
6 nested libtest block(s) excluded (child output of tests that re-execute the test binary).

### `user-profile-lib-tests` (exit 0)
| Target | Declared tests | Result line (verbatim) |
|---|---|---|
| `unittests src/lib.rs` | 9 | `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 232 filtered out; finished in 12.22s` |

Sum over top-level result lines: 9 passed, 0 failed, 0 ignored, 232 filtered out.

### `rustc-wrapper-run-daemon-tests` (exit 0)
| Target | Declared tests | Result line (verbatim) |
|---|---|---|
| `unittests src/lib.rs` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 48 filtered out; finished in 0.00s` |
| `unittests src/bin/mantle-cc-driver.rs` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |
| `unittests src/bin/mantle-rust-cache-daemon.rs` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s` |
| `unittests src/bin/mantle-rust-wait-wrapper.rs` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |
| `unittests src/bin/mantle-rustc-manifest.rs` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s` |
| `unittests src/bin/mantle-rustc-wrapper.rs` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |

Sum over top-level result lines: 0 passed, 0 failed, 0 ignored, 53 filtered out.

### `family-parity-cli-tests` (exit 0)
| Target | Declared tests | Result line (verbatim) |
|---|---|---|
| `tests/bootstrap_parity_cli.rs` | 21 | `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.37s` |
| `tests/bootstrap_validate_cli.rs` | 5 | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.54s` |
| `tests/cargo_free_self_build_cli.rs` | 15 | `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.73s` |
| `tests/kernel_bundle_oci_cli.rs` | 4 | `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s` |
| `tests/kernel_bundle_oci_registry_cli.rs` | 11 | `test result: ok. 10 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 2.24s` |
| `tests/operator_diagnostics.rs` | 30 | `test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.98s` |
| `tests/project_build_smoke.rs` | 19 | `test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 9.71s` |
| `tests/project_cli.rs` | 22 | `test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.46s` |
| `tests/project_refresh_cli.rs` | 14 | `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.33s` |
| `tests/release_cli.rs` | 154 | `test result: ok. 154 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.25s` |
| `tests/remote_credentials_cli.rs` | 9 | `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.50s` |
| `tests/remote_stdio_cli.rs` | 8 | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.99s` |
| `tests/rust_plan_cli.rs` | 59 | `test result: ok. 59 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 98.46s` |
| `tests/source_bundle_cli.rs` | 9 | `test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.72s` |
| `tests/source_bundle_hydration_cli.rs` | 2 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s` |
| `tests/store_archive_cli.rs` | 14 | `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 143.00s` |
| `tests/store_gc_cli.rs` | 20 | `test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 15.34s` |
| `tests/transcript_cli.rs` | 14 | `test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.32s` |

Sum over top-level result lines: 429 passed, 0 failed, 1 ignored, 0 filtered out.

### `evaluation-and-example-tests` (exit 0)
| Target | Declared tests | Result line (verbatim) |
|---|---|---|
| `tests/causal_build_trace.rs` | 3 | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.03s` |
| `tests/evaluator_budget_cli.rs` | 13 | `test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.51s` |
| `tests/example_projects.rs` | 37 | `test result: ok. 32 passed; 0 failed; 5 ignored; 0 measured; 0 filtered out; finished in 50.72s` |
| `tests/integration.rs` | 81 | `test result: ok. 81 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 38.64s` |

Sum over top-level result lines: 129 passed, 0 failed, 5 ignored, 0 filtered out.

### Commands that executed zero tests
Exit status 0 with no executed tests is not verification. These commands are not counted as test evidence:

- `rustc-wrapper-run-daemon-tests`: 0 passed, 0 failed, 0 ignored, 53 filtered out; argv `cargo test --locked --offline -p crunch-rustc-wrapper run_daemon -- --test-threads 1`.

The I6 handoff filter `run_daemon` is a function name, not a test name. The unfiltered package run below executes the crate's tests instead.

## Unfiltered `crunch-rustc-wrapper` package tests
- Runner: `run-commands.py` (BLAKE3 `93e431b88e990538ee266521a2ade108881d0b5d51f73b32f94ba461acd67b3d`), request BLAKE3 `0cd46e5409954c783afa0236b5c433b51ce85a1d9c83733b68008b11316a48ad`.
- Retained run directory: `/home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/thin-cli-wrapper-unfiltered-v89`.
- Started 2026-10-30T00:03:10.055919-04:00, finished 2026-10-30T00:03:17.702681-04:00; driver exit 0; selected-subject digests unchanged: True.
| # | Label | Exit | Elapsed | stdout BLAKE3 (first 16) | stderr BLAKE3 (first 16) |
|---|---|---|---|---|---|
| 1 | `source-identity` | 0 | 0.0s | `05fd3aae0655d553` | `af1349b9f5f9a1a6` |
| 2 | `rustc-wrapper-package-tests` | 0 | 7.4s | `f355204d1af710b4` | `97ae78dca82bf6ac` |
| 3 | `worktree-status-after` | 0 | 0.1s | `af1349b9f5f9a1a6` | `af1349b9f5f9a1a6` |

Exact argv:
- `source-identity`: `git rev-parse HEAD HEAD^{tree}`
- `rustc-wrapper-package-tests`: `cargo test --locked --offline -p crunch-rustc-wrapper -- --test-threads 1`
- `worktree-status-after`: `git status --porcelain`

| Target | Declared tests | Result line (verbatim) |
|---|---|---|
| `unittests src/lib.rs` | 48 | `test result: ok. 48 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 5.74s` |
| `unittests src/bin/mantle-cc-driver.rs` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |
| `unittests src/bin/mantle-rust-cache-daemon.rs` | 3 | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |
| `unittests src/bin/mantle-rust-wait-wrapper.rs` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |
| `unittests src/bin/mantle-rustc-manifest.rs` | 2 | `test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |
| `unittests src/bin/mantle-rustc-wrapper.rs` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |
| `doc-tests crunch_rustc_wrapper` | 0 | `test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` |

Sum over top-level result lines: 53 passed, 0 failed, 0 ignored, 0 filtered out.

## Family map (name-based, not an audit)
The map groups the commanded test targets by the families named in the I6 handoff. It is based on target names; it does not assert that a target exercises every operation of its family.

| Family | Targets (declared / passed / failed / ignored) |
|---|---|
| diagnostics/operator output | `operator_diagnostics`: 30 / 30 / 0 / 0 |
| build / plan / evaluation | `integration`: 81 / 81 / 0 / 0; `project_build_smoke`: 19 / 19 / 0 / 0; `causal_build_trace`: 3 / 3 / 0 / 0; `evaluator_budget_cli`: 13 / 13 / 0 / 0 |
| Rust-plan / Cargo-free self-build | `rust_plan_cli`: 59 / 59 / 0 / 0; `cargo_free_self_build_cli`: 15 / 15 / 0 / 0 |
| remote-build | `remote_credentials_cli`: 9 / 9 / 0 / 0; `remote_stdio_cli`: 8 / 8 / 0 / 0 |
| store | `store_gc_cli`: 20 / 20 / 0 / 0; `store_archive_cli`: 14 / 14 / 0 / 0 |
| source | `source_bundle_cli`: 9 / 9 / 0 / 0; `source_bundle_hydration_cli`: 2 / 2 / 0 / 0 |
| release | `release_cli`: 154 / 154 / 0 / 0 |
| project | `project_cli`: 22 / 22 / 0 / 0; `project_refresh_cli`: 14 / 14 / 0 / 0; `example_projects`: 37 / 32 / 0 / 5 |
| bootstrap | `bootstrap_validate_cli`: 5 / 5 / 0 / 0; `bootstrap_parity_cli`: 21 / 21 / 0 / 0; `transcript_cli`: 14 / 14 / 0 / 0 |
| artifact | `kernel_bundle_oci_cli`: 4 / 4 / 0 / 0; `kernel_bundle_oci_registry_cli`: 11 / 10 / 0 / 1 |

## Capability-gated skips (no-capture re-runs)
Capability-gated tests print a skip line (`skipping: ...` or `SKIP: ...`) and return early; without `--nocapture` such a test still reports `ok`. Targets using either idiom were re-run with `--nocapture`, and stderr lines containing the word `skip` or `skipping` (case-insensitive) are attributed to the preceding cargo `Running` line. Targets were chosen by searching `tests/*.rs` for those idioms and for `can_build`/`has_bwrap` gate helpers; a search of the other parity targets found no capability gate (their early `return;` statements are child-mode dispatch, optional-environment paths or fake-binary mains).

### Run `thin-cli-skip-visibility-v88`
- Runner: `run-commands.py` (BLAKE3 `93e431b88e990538ee266521a2ade108881d0b5d51f73b32f94ba461acd67b3d`), request BLAKE3 `cee7f4fb6e4e8af70fcbec6fc45ede2fae188f7698e1393f6283b3a9e0674fb6`.
- Retained run directory: `/home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/thin-cli-skip-visibility-v88`.
- Started 2026-10-29T23:59:48.537682-04:00, finished 2026-10-30T00:01:27.607774-04:00; driver exit 0; selected-subject digests unchanged: True.
| # | Label | Exit | Elapsed | stdout BLAKE3 (first 16) | stderr BLAKE3 (first 16) |
|---|---|---|---|---|---|
| 1 | `source-identity` | 0 | 0.0s | `05fd3aae0655d553` | `af1349b9f5f9a1a6` |
| 2 | `prerequisite-probe` | 0 | 0.4s | `b61d34a1fa0d331e` | `af1349b9f5f9a1a6` |
| 3 | `capability-gated-targets-nocapture` | 0 | 98.0s | `90912f88b3f72265` | `1476c1314482db12` |
| 4 | `worktree-status-after` | 0 | 0.2s | `af1349b9f5f9a1a6` | `af1349b9f5f9a1a6` |

Exact argv:
- `source-identity`: `git rev-parse HEAD HEAD^{tree}`
- `prerequisite-probe`: `sh -c set +e; bwrap --ro-bind / / -- /bin/true; echo "bwrap-ro-bind-probe-exit=$?"; test -d /nix/store && echo "nix-store-present=yes" || echo "nix-store-present=no"; ls /nix/store | head -c 0; echo "busybox-sandbox-shell=${SNIX_BUILD_SANDBOX_SHELL}"; test -x "${SNIX_BUILD_SANDBOX_SHELL}" && echo "sandbox-shell-executable=yes" || echo "sandbox-shell-executable=no"; exit 0`
- `capability-gated-targets-nocapture`: `cargo test --locked --offline -p mantle --test bootstrap_validate_cli --test causal_build_trace --test integration --test project_build_smoke --test store_gc_cli -- --test-threads 1 --nocapture`
- `worktree-status-after`: `git status --porcelain`

Prerequisite probe (verbatim):
```
bwrap-ro-bind-probe-exit=0
nix-store-present=yes
busybox-sandbox-shell=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
sandbox-shell-executable=yes
```

| Target | Declared tests | Result line (verbatim) | skip-marked stderr lines |
|---|---|---|---|
| `tests/bootstrap_validate_cli.rs` | 5 | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.06s` | 0 |
| `tests/causal_build_trace.rs` | 3 | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.42s` | 0 |
| `tests/integration.rs` | 81 | `test result: ok. 81 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 29.41s` | 0 |
| `tests/project_build_smoke.rs` | 19 | `test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 21.24s` | 0 |
| `tests/store_gc_cli.rs` | 20 | `test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 42.15s` | 0 |

Skip-marked stderr lines by target (verbatim):

- none observed in the retained no-capture stderr

### Run `thin-cli-example-projects-skip-v90`
- Runner: `run-commands.py` (BLAKE3 `93e431b88e990538ee266521a2ade108881d0b5d51f73b32f94ba461acd67b3d`), request BLAKE3 `8aba378cddfc76ef9496f561859bd56130b841add65b4d0fb0c39be987b40790`.
- Retained run directory: `/home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/thin-cli-example-projects-skip-v90`.
- Started 2026-10-30T00:03:57.424015-04:00, finished 2026-10-30T00:04:49.994024-04:00; driver exit 0; selected-subject digests unchanged: True.
| # | Label | Exit | Elapsed | stdout BLAKE3 (first 16) | stderr BLAKE3 (first 16) |
|---|---|---|---|---|---|
| 1 | `source-identity` | 0 | 0.0s | `05fd3aae0655d553` | `af1349b9f5f9a1a6` |
| 2 | `capability-gated-targets-nocapture` | 0 | 52.2s | `adebe058085f409c` | `571f0b84d994e75d` |
| 3 | `worktree-status-after` | 0 | 0.1s | `af1349b9f5f9a1a6` | `af1349b9f5f9a1a6` |

Exact argv:
- `source-identity`: `git rev-parse HEAD HEAD^{tree}`
- `capability-gated-targets-nocapture`: `cargo test --locked --offline -p mantle --test example_projects -- --test-threads 1 --nocapture`
- `worktree-status-after`: `git status --porcelain`

| Target | Declared tests | Result line (verbatim) | skip-marked stderr lines |
|---|---|---|---|
| `tests/example_projects.rs` | 37 | `test result: ok. 32 passed; 0 failed; 5 ignored; 0 measured; 0 filtered out; finished in 50.86s` | 0 |

Skip-marked stderr lines by target (verbatim):

- none observed in the retained no-capture stderr

Totals: `thin-cli-skip-visibility-v88`: 128 tests reported passed or failed, 0 skip-marked stderr lines; `thin-cli-example-projects-skip-v90`: 32 tests reported passed or failed, 0 skip-marked stderr lines.

## Direct CLI runtime smoke and evaluation budget reports
- Runner: `run-commands.py` (BLAKE3 `93e431b88e990538ee266521a2ade108881d0b5d51f73b32f94ba461acd67b3d`), request BLAKE3 `078d73c691b7942e8fbe1565f1f6e636cdad3ecfb70ed3137d6a6f88815921bd`.
- Retained run directory: `/home/brittonr/.cargo-target/mantle-complete-20261026-9ar4jfgk/thin-cli-smoke-v86`.
- Started 2026-10-29T23:09:47.177721-04:00, finished 2026-10-29T23:09:49.310681-04:00; driver exit 0; selected-subject digests unchanged: True.
| # | Label | Exit | Elapsed | stdout BLAKE3 (first 16) | stderr BLAKE3 (first 16) |
|---|---|---|---|---|---|
| 1 | `source-identity` | 0 | 0.0s | `05fd3aae0655d553` | `af1349b9f5f9a1a6` |
| 2 | `mantle-binary-identity` | 0 | 1.1s | `6c41afa8a16d7386` | `af1349b9f5f9a1a6` |
| 3 | `direct-cli-runtime-smoke` | 0 | 0.6s | `c96ffc24c99dcf84` | `af1349b9f5f9a1a6` |
| 4 | `eval-positive` | 0 | 0.1s | `ac935e8f4dc1b4c4` | `af1349b9f5f9a1a6` |
| 5 | `eval-strict-budget` | 0 | 0.1s | `ac935e8f4dc1b4c4` | `af1349b9f5f9a1a6` |
| 6 | `eval-observe-budget` | 0 | 0.1s | `ac935e8f4dc1b4c4` | `af1349b9f5f9a1a6` |
| 7 | `worktree-status-after` | 0 | 0.1s | `af1349b9f5f9a1a6` | `af1349b9f5f9a1a6` |

Smoke subject: commit `5fc2ad4b3dfb1ae80ff5060cdd8fb6670230e316`, tree `35d06fef314555b7df2e47d1f15294caa29a47ad`; `mantle` binary BLAKE3 `b599e0ca5ad3b6bd6327bcb2675e5e62b6b420063a9cc6f5cf5839beafc9e40d`; smoke script BLAKE3 `510a4aa8dde0ea7dab0b441777429810a23eae5ac58619d171617e1ef39b2311` (extracted verbatim from the I6 handoff).

### `direct-cli-runtime-smoke` (exit 0), stdout verbatim
```
refactor False plan 0 refactor session: crunch-to-mantle-project-identity
root: /tmp/mantle-i6-smoke-yogv_65x/human
dry-run: true
operations:
  rename-file: crunch-project.ncl -> mantle-project.ncl

refactor False apply 0 refactor session: crunch-to-mantle-project-identity
root: /tmp/mantle-i6-smoke-yogv_65x/human
dry-run: false
operations:
  rename-file: crunch-project.ncl -> mantle-project.ncl

refactor True plan 0 {
  "schema": "mantle-refactor-plan-v1",
  "session_id": "crunch-to-mantle-project-identity",
  "root": "/tmp/mantle-i6-smoke-yogv_65x/json",
  "dry_run": true,
  "operations": [
    {
      "kind": "rename-file",
      "from": "crunch-project.ncl",
      "to": "mantle-project.ncl",
      "apply_supported": true
    }
  ],
  "diagnostics": []
}

refactor True apply 0 {
  "schema": "mantle-refactor-plan-v1",
  "session_id": "crunch-to-mantle-project-identity",
  "root": "/tmp/mantle-i6-smoke-yogv_65x/json",
  "dry_run": false,
  "operations": [
    {
      "kind": "rename-file",
      "from": "crunch-project.ncl",
      "to": "mantle-project.ncl",
      "apply_supported": true
    }
  ],
  "diagnostics": []
}

adapter-fault 3  {"error":"refactor I/O error at /tmp/mantle-i6-smoke-yogv_65x/not-a-directory/crunch-project.ncl: Not a directory (os error 20)","code":3,"kind":"internal"}

report-baseline raw-descriptors 0
render-fault raw-descriptors 3 error: writing operator command descriptors: No space left on device (os error 28)

report-baseline catalog 0
render-fault catalog 3 error: writing operator command catalog: No space left on device (os error 28)

report-baseline reference 0
render-fault reference 3 error: writing operator command reference: No space left on device (os error 28)

report-baseline workflow 0
render-fault workflow 3 error: writing operator workflow: No space left on device (os error 28)

report-baseline check 0
render-fault check 3 error: writing operator command contract check: No space left on device (os error 28)

gateway-binding 1 is-valid-path 0 {"operation":"store-read","request_digest":[19,216,237,137,245,167,181,85,88,127,22,77,202,157,8,155,82,39,231,104,126,134,140,186,60,252,86,88,110,57,241,247],"target":"/nix/store/00000000000000000000000000000000-fixture"}

gateway-binding 2 is-valid-path 0 {"operation":"store-read","request_digest":[225,112,228,34,9,101,139,137,141,164,121,71,166,220,92,135,113,31,115,239,153,120,67,232,227,181,214,138,159,222,121,5],"target":"/nix/store/00000000000000000000000000000000-fixture"}

gateway-binding 1 query-path-info 0 {"operation":"store-read","request_digest":[69,123,240,122,226,10,190,127,100,41,118,199,61,130,42,40,227,65,80,229,217,171,50,196,172,135,247,76,98,221,222,169],"target":"/nix/store/00000000000000000000000000000000-fixture"}

gateway-zero-generation 3  error: gateway-host-unavailable

direct-cli-smoke: all assertions held
```
stderr: empty

### Evaluation budget reports versus execution mode
| Run | Policy file | Policy `mode` | Report `mode` | `terminal_disposition` | metrics | exit |
|---|---|---|---|---|---|---|
| `eval-strict-budget` | `config/evaluation/default-policy.json` | `enforce` (`mantle-evaluation-strict-default-v1`) | `enforce` | `success` | 16 | 0 |
| `eval-observe-budget` | `fixtures/evaluation-budget/policy-observe-only.json` | `observe-only` (`mantle-evaluation-observe-fixture-v1`) | `observe-only` | `success` | 13 | 0 |

Persisted reports (BLAKE3): `strict-report.json` `dec718e152f6148738b21662c1ed447a1b851e95cf191af8347db1cb413119cb`, `observe-report.json` `56d4967beebaf5f54ace64c48a78003f30fc3eeca4f4ddbae48a508167eb0404`.

`eval-positive` stdout (verbatim):
```
{
  "answer": 42,
  "message": "bounded"
}
```

`eval-strict-budget` stdout (verbatim):
```
{
  "answer": 42,
  "message": "bounded"
}
```

`eval-observe-budget` stdout (verbatim):
```
{
  "answer": 42,
  "message": "bounded"
}
```

## Non-claims
- Passing targets show the existing tests passed on this tree under serialized execution; they do not show that every one of the 154 catalog leaves or every operation branch is covered by a test.
- A capability-gated test that skipped is not execution evidence; the skip counts above are the recorded limit of what the gated targets exercised.
- 99 tests were `#[ignore]`d in the parity run and did not run (per-target counts are in the tables above); no ignored test is evidence here.
- A command that exits 0 while executing zero tests is not verification; any such command is called out above and is not counted.
- The smoke exercises refactor plan/apply, the adapter-fault and fallible-stdout exits, operator-contract render faults and gateway binding determinism on one binary; it is not an authenticated remote service, cold-proof or source-built eligibility observation.
- Budget reports were compared with their execution mode by field equality; compliance with a budget does not prove evaluator correctness, reproducibility, build correctness or release eligibility.
- No flake check, Clippy, formatting or Cairn gate result is claimed by this file (V5).
- No accepted-spec sync, archive, release eligibility or successor publication follows.
