## Phase 1: Sandbox hardening (bwrap)

- [ ] Add `--hostname localhost` to `COMMON_BWRAP_ARGS` in `vendor/snix-build/src/bwrap/mod.rs`
- [ ] Add `/proc` masking args (`--ro-bind-try /dev/null /proc/cpuinfo` etc.) after `--proc /proc`
- [ ] Add `/dev/random` and `/dev/urandom` masking args (`--ro-bind /dev/null /dev/random` etc.) after `--dev /dev`
- [ ] Add `--unshare-cgroup-try` to `COMMON_BWRAP_ARGS`
- [ ] Add unit test: assert `COMMON_BWRAP_ARGS` contains `--hostname`, proc masks, dev masks, cgroup unshare

## Phase 2: Build environment defaults

- [ ] Change `NIX_BUILD_CORES` default from `"0"` to `"1"` in `SANDBOX_ENV_VARS` (`crates/crunch-build/src/build_request.rs`)
- [ ] Update unit test `build_request_has_sandbox_env_vars` to assert `NIX_BUILD_CORES=1`
- [ ] Update bootstrap derivations that need parallelism (`bootstrap/*.ncl`) to set `NIX_BUILD_CORES` explicitly

## Phase 3: Output normalization in export

- [ ] Add `filetime` crate to `crates/crunch-store/Cargo.toml`
- [ ] Set non-executable file permissions to `0o444` in `export_file_to_disk`
- [ ] Set directory permissions to `0o555` after `create_dir_all` in `export_castore_to_disk`
- [ ] Set mtime to Unix timestamp `1` on all exported files and directories
- [ ] Set lmtime on exported symlinks via `filetime::set_symlink_file_times`
- [ ] Add unit test: exported non-executable file has mode `0o444`
- [ ] Add unit test: exported directory has mode `0o555`
- [ ] Add unit test: exported file has mtime `1`
- [ ] Add unit test: exported symlink has lmtime `1`

## Phase 4: Orchestrator map ordering

- [ ] Replace `output_infos: HashMap<String, PathInfo>` with `BTreeMap` in `BuildOutcome` and `finish_build`
- [ ] Replace `substitutions: HashMap<String, OutputSubstitutionReport>` with `BTreeMap` in `BuildOutcome`
- [ ] Audit remaining `HashMap` uses in `orchestrate.rs`; convert any that are iterated into output
- [ ] Add assertion: `BuildOutcome.outputs` iteration order matches sorted output names

## Phase 5: Validation

- [ ] Run `cargo test -p snix-build -p crunch-build -p crunch-store -p crunch-pipeline` and verify all pass
- [ ] Run self-hosting proof (`cargo test -p crunch --test self_hosting -- --ignored --nocapture`) or verify it compiles and lists
- [ ] Verify bootstrap builds succeed with `NIX_BUILD_CORES=1` (run `crunch self-build --store /tmp/sandbox-determinism-test --no-substitute -j 4`)
