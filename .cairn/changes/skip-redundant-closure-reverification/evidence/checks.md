# Checks: skip redundant closure re-verification

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
| `cargo test -p crunch-store --lib` (includes `handle::tests::cached_path_info_returns_a_complete_local_path_without_exporting`) | `471 passed; 1 failed` — the single failure was the environment `SUN_LEN` test `query::tests::store_verify_read_failure_does_not_persist_pathinfo`, which passes alone with `TMPDIR=/tmp/mf-tmp`. |
| `cargo test -p mantle --test cache_inputs_build_route --test foreign_import_cli` | `cache_inputs_build_route`: `test result: ok. 19 passed; 0 failed`; `foreign_import_cli`: `test result: ok. 18 passed; 0 failed; 1 ignored`. |
| `cargo fmt --check -p mantle -p crunch-store -p crunch-build -p crunch-pipeline` | clean: no diff lines |
| `cargo clippy -p mantle -p crunch-store -p crunch-build -p crunch-pipeline --all-targets --no-deps -- -D warnings` | `Finished `dev` profile [unoptimized + debuginfo] target(s) in 4m 35s` with no warning or error lines. |
| `cargo test -p crunch-store --lib -- pull::tests::http_closure_content_admission handle::tests::cached_path_info pull::tests::http_nar_refusal_does_not_echo_cache_credentials` | `test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 468 filtered out; finished in 0.33s` (covers the restored lookup test) |
| `cargo test -p mantle --bin mantle -- foreign_` | `test result: ok. 122 passed; 0 failed; 1 ignored` |
| `git diff --check` | exit 0, no output |

## Cairn

| Command | Result |
|---|---|
| `cairn validate --root .` | `"issues": []`, `"valid": true` |
| `cairn gate proposal skip-redundant-closure-reverification --root .` | `"verdict": "PASS"` |
| `cairn gate design skip-redundant-closure-reverification --root .` | `"verdict": "PASS"` |
| `cairn gate tasks skip-redundant-closure-reverification --root .` | `"verdict": "PASS"` |

## Old-code demonstration

`cached_path_info` was temporarily given the pre-change physical-export step (the same call
`export_cached_path_info` makes) and the new unit test was run once:

```
thread 'handle::tests::cached_path_info_returns_a_complete_local_path_without_exporting' panicked
  at crates/crunch-store/src/handle.rs:4833:9: lookup must not materialize the physical export
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 471 filtered out; finished in 0.02s
```

The edit was reverted; the final run is `test result: ok. 1 passed`. On `origin/main` the method and
its test do not exist, so the test also could not run at all.
