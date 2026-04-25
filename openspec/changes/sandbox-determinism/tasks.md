## Phase 1: Sandbox hardening (bwrap)

- [x] Add `--hostname localhost` to `COMMON_BWRAP_ARGS` in `vendor/snix-build/src/bwrap/mod.rs` ✅ 5m
- [x] Add `/proc` masking args (`--ro-bind-try /dev/null /proc/cpuinfo` etc.) after `--proc /proc` ✅ 3m
- [x] Add `/dev/random` and `/dev/urandom` masking args (`--ro-bind /dev/null /dev/random` etc.) after `--dev /dev` ✅ 3m
- [x] Add `--tmpfs /dev/shm` after `--dev /dev` to isolate shared memory per build ✅ 1m
- [x] Add `--unshare-cgroup-try` to `COMMON_BWRAP_ARGS` ✅ 2m
- [x] Verify no code path mounts host `/sys` into the sandbox (bwrap root is `--tmpfs /` so `/sys` should be absent; add assertion test) ✅ 3m
- [x] Replace host `/etc/resolv.conf` and `/etc/services` bind-mounts for network-enabled builds with synthetic files written alongside existing `/etc/passwd`, `/etc/group`, `/etc/hosts` ✅ 8m
- [x] Add unit test: assert `COMMON_BWRAP_ARGS` contains `--hostname`, proc masks, dev masks, `/dev/shm` isolation, cgroup unshare ✅ 5m
- [x] Add unit test: network-enabled sandbox uses synthetic `resolv.conf`, not host bind-mount ✅ 2m

## Phase 2: Build environment defaults

- [x] Change `NIX_BUILD_CORES` default from `"0"` to `"1"` in `SANDBOX_ENV_VARS` (`crates/crunch-build/src/build_request.rs`) ✅ 3m
- [x] Update unit test `build_request_has_sandbox_env_vars` to assert `NIX_BUILD_CORES=1` ✅ 2m
- [x] Update bootstrap derivations that need parallelism (`bootstrap/*.ncl`) to set `NIX_BUILD_CORES` explicitly ✅ 4m

## Phase 3: Output normalization in export

- [x] Add `filetime` crate to `crates/crunch-store/Cargo.toml` ✅ 2m
- [x] Set non-executable file permissions to `0o444` in `export_file_to_disk` ✅ 8m
- [x] Set directory permissions to `0o555` after `create_dir_all` in `export_castore_to_disk` ✅ 8m
- [x] Set mtime to Unix timestamp `1` on all exported files and directories ✅ 10m
- [x] Set lmtime on exported symlinks via `filetime::set_symlink_file_times` ✅ 4m
- [x] Add unit test: exported non-executable file has mode `0o444` ✅ 3m
- [x] Add unit test: exported directory has mode `0o555` ✅ 3m
- [x] Add unit test: exported file has mtime `1` ✅ 3m
- [x] Add unit test: exported symlink has lmtime `1` ✅ 3m

## Phase 4: Orchestrator map ordering

- [ ] Replace `output_infos: HashMap<String, PathInfo>` with `BTreeMap` in `BuildOutcome` and `finish_build`
- [ ] Replace `substitutions: HashMap<String, OutputSubstitutionReport>` with `BTreeMap` in `BuildOutcome`
- [ ] Audit remaining `HashMap` uses in `orchestrate.rs`; convert any that are iterated into output
- [ ] Add assertion: `BuildOutcome.outputs` iteration order matches sorted output names

## Phase 5: Validation

- [ ] Run `cargo test -p snix-build -p crunch-build -p crunch-store -p crunch-pipeline` and verify all pass
- [ ] Run self-hosting proof (`cargo test -p crunch --test self_hosting -- --ignored --nocapture`) or verify it compiles and lists
- [ ] Verify bootstrap builds succeed with `NIX_BUILD_CORES=1` (run `crunch self-build --store /tmp/sandbox-determinism-test --no-substitute -j 4`)
- [ ] Verify FOD fetches succeed with synthetic `resolv.conf` (run a `crunch.fetchurl` build with a network URL)

## Deferred (documented but out of scope)

- Seccomp syscall filtering (separate change with its own compatibility surface)
- Time namespace (`--unshare-time`, kernel 5.6+, bwrap lacks support)
- Full `/proc` virtualization via FUSE-based procfs filter
