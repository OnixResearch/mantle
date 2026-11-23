# Checks: verify each castore node's completeness once per store handle

Environment for every command (from the follow-up spec):

```
export PATH=/nix/store/mvj6y8k3xs35cj7kq937xi44q4hl3r93-bubblewrap-0.12.0/bin:/nix/store/2msw5nclhpsm80z65dqmjiv8kkras989-clang-wrapper-21.1.8/bin:$PATH
export RUSTC_WRAPPER= SNIX_BUILD_SANDBOX_SHELL=/bin/sh CARGO_INCREMENTAL=0
export CARGO_TARGET_DIR=/datapool/mantle-store/followups-20261122/mantle-target
export TMPDIR=/datapool/mantle-store/followups-20261122/mantle-tmp
```

## Cargo

| Command | Result | Transcript |
|---|---|---|
| `cargo fmt --check -p mantle -p crunch-store -p crunch-build -p crunch-pipeline` | `exit=0`, no diff lines | `fmt.log` |
| `cargo clippy -p mantle -p crunch-store -p crunch-build -p crunch-pipeline --all-targets --no-deps -- -D warnings` | `Finished `dev` profile [unoptimized + debuginfo] target(s) in 4m 39s`, `exit=0`, no warning or error lines | `clippy.log` |
| `cargo test -p crunch-store --lib` | Final clean run: `TMPDIR=/tmp/mf-tmp cargo test -p crunch-store --lib -- --test-threads 1` → `test result: ok. 476 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 531.97s`, `exit=0` | `crunch-store-lib.log` |
| `cargo test -p mantle --test cache_inputs_build_route --test foreign_import_cli` | `cache_inputs_build_route`: `test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.34s`; `foreign_import_cli`: `test result: ok. 18 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 4.95s`; `exit=0` | `route-suites.log` |
| `cargo test -p mantle --bin mantle -- foreign_` | `test result: ok. 122 passed; 0 failed; 1 ignored; 0 measured; 2907 filtered out; finished in 89.07s`, `exit=0` | `foreign-bin.log` |
| `git diff --check` | `exit=0`, no output | `git-diff-check.log` |

The first `crunch-store` lib run used the long `TMPDIR` above and failed only
`query::tests::store_verify_read_failure_does_not_persist_pathinfo` with
`Error { kind: InvalidInput, message: "path must be shorter than SUN_LEN" }` (475 passed, 1 failed).
That is the known long-TMPDIR environment failure, not this change: the test binds a Unix socket
under `TMPDIR`. The clean rerun uses the short `TMPDIR=/tmp/mf-tmp` (serial, `--test-threads 1`, to
avoid fixture resource contention) and is recorded in `crunch-store-lib.log`; the long-TMPDIR run is
kept as `crunch-store-lib-env-fail.log`.

## New unit tests

All four are in `crates/crunch-store/src/handle.rs` and passed in the lib run above:

- `handle::tests::cached_path_info_repeat_on_one_handle_reads_no_blobs` — first lookup reads blobs,
  the repeat performs zero blob reads and zero probes and returns the same `PathInfo`, and a
  different node is still checked in full.
- `handle::tests::castore_node_verified_complete_rechecks_incomplete_nodes` — a size-mismatched
  node returns false on every call, reads the blob every time, and is never recorded.
- `handle::tests::castore_node_verified_complete_rechecks_an_absent_blob` — an absent blob returns
  false on every call, probes the services every time, and is never recorded.
- `handle::tests::garbage_collection_clears_verified_complete_nodes` — GC empties the memo and the
  next check reads the services again.

## Cairn

Pinned CLI: `/nix/store/1z6bhr2cl43z4kpsvjkxy9b47nqr9pkg-cairn-0.1.0/bin/cairn`.

| Command | Result | Transcript |
|---|---|---|
| `cairn validate --root .` | `"issues": []`, `"valid": true` | `cairn-validate.log` |
| `cairn gate proposal verify-castore-completeness-once --root .` | `"verdict": "PASS"`, `"issues": []` | `cairn-gate-proposal.log` |
| `cairn gate design verify-castore-completeness-once --root .` | `"verdict": "PASS"`, `"issues": []` | `cairn-gate-design.log` |
| `cairn gate tasks verify-castore-completeness-once --root .` | `"verdict": "PASS"`, `"issues": []` | `cairn-gate-tasks.log` |

## Live finding

See `finding.md`.
