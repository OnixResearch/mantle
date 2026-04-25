## Phase 1: Sandbox hardening (bwrap)

- [x] Add `--hostname localhost` to `COMMON_BWRAP_ARGS` in `vendor/snix-build/src/bwrap/mod.rs` ✅ 5m
- [x] Add `/proc` masking args for `/proc/cpuinfo`, `/proc/meminfo`, `/proc/stat`, `/proc/loadavg`, `/proc/uptime`, and `/proc/version` after `--proc /proc` ✅ 3m
- [x] Add `/dev/random` and `/dev/urandom` masking args (`--ro-bind /dev/null /dev/random` etc.) after `--dev /dev` ✅ 3m
- [x] Add `--tmpfs /dev/shm` after `--dev /dev` to isolate shared memory per build ✅ 1m
- [x] Add `--unshare-cgroup-try` to `COMMON_BWRAP_ARGS` ✅ 2m
- [x] Verify no code path mounts host `/sys` into the sandbox (bwrap root is `--tmpfs /` so `/sys` should be absent; add assertion test) ✅ 3m
- [x] Replace host `/etc/resolv.conf` and `/etc/services` bind-mounts for network-enabled builds with synthetic files written alongside existing `/etc/passwd`, `/etc/group`, `/etc/hosts` ✅ 8m
- [x] Add unit test: assert `COMMON_BWRAP_ARGS` contains `--hostname`, exact proc masks, dev masks, `/dev/shm` isolation, cgroup unshare, and no `/proc/self` mask ✅ 7m
- [x] Add unit test: network-enabled sandbox uses exact synthetic `resolv.conf` and `services` contents, not host bind-mounts ✅ 4m
- [x] Add unit test: non-network sandbox omits `/etc/resolv.conf` and `/etc/services` while unsharing network ✅ 3m

## Phase 2: Build environment defaults

- [x] Change `NIX_BUILD_CORES` default from `"0"` to `"1"` in `SANDBOX_ENV_VARS` (`crates/crunch-build/src/build_request.rs`) ✅ 3m
- [x] Update unit test `build_request_has_sandbox_env_vars` to assert `NIX_BUILD_CORES=1` ✅ 2m
- [x] Add unit tests: `NIX_BUILD_CORES` override uses derivation value without audit, and `SOURCE_DATE_EPOCH` override stays allowed without audit in Practical and Strict modes ✅ 5m
- [x] Update bootstrap derivations that need parallelism (`bootstrap/*.ncl`) to set `NIX_BUILD_CORES` explicitly ✅ 4m

## Phase 3: Output normalization in export

- [x] Add `filetime` crate to `crates/crunch-store/Cargo.toml` ✅ 2m
- [x] Set non-executable file permissions to `0o444` in `export_file_to_disk` ✅ 8m
- [x] Set directory permissions to `0o555` after `create_dir_all` in `export_castore_to_disk` ✅ 8m
- [x] Set mtime to Unix timestamp `1` on all exported files and directories, finalizing directory mtimes after child entries are written ✅ 10m
- [x] Set lmtime on exported symlinks via `filetime::set_symlink_file_times` ✅ 4m
- [x] Add unit test: exported non-executable file has mode `0o444` ✅ 3m
- [x] Add unit test: exported executable file has mode `0o555` ✅ 3m
- [x] Add unit test: exported directory has mode `0o555` ✅ 3m
- [x] Add unit test: exported file has mtime `1` ✅ 3m
- [x] Add unit test: exported directory has mtime `1` ✅ 3m
- [x] Add unit test: exported symlink has lmtime `1` ✅ 3m

## Phase 4: Orchestrator map ordering

- [x] Replace `output_infos: HashMap<String, PathInfo>` with `BTreeMap` in `BuildOutcome` and `finish_build` ✅ 8m
- [x] Replace `substitutions: HashMap<String, OutputSubstitutionReport>` with `BTreeMap` in `BuildOutcome` ✅ 4m
- [x] Audit remaining `HashMap` uses in `orchestrate.rs` and `BuildOutcome` consumers; convert any maps iterated into build outputs, attestation/reporting inputs, or user-visible output ✅ 6m
- [x] Add assertion: `BuildOutcome.outputs` iteration order matches sorted output names ✅ 4m

## Phase 5: Validation

- [x] Run `cargo test -p snix-build -p crunch-build -p crunch-store -p crunch-pipeline` and verify all pass ✅ pueue#96 8s
- [x] Run focused bwrap hardening tests (`cargo test -p snix-build --lib -- bwrap::tests`) ✅ 16s
- [x] Run ambient-state determinism regression coverage varying `HOME`, `PATH`, `USER`, `TZ`, `LANG`, `TMPDIR`, current working directory, and umask; compare digest/audit or blocker stability ✅ covered by `crunch-pipeline` integration tests in pueue#88
- [x] Fix self-hosting proof invalidation helper so read-only normalized `*-crunch` outputs can be removed before stage2 ✅ pueue#102 2s
- [x] Run full self-hosting proof (`./scripts/prove-self-hosting.sh --bundle-dir target/self-hosting-proof/sandbox-determinism-rerun`) ✅ pueue#103 35m9s
- [x] Verify bootstrap builds succeed with `NIX_BUILD_CORES=1` (run `crunch self-build --store /tmp/sandbox-determinism-test --no-substitute -j 4`) ✅ pueue#93 18m29s
- [x] Verify FOD fetches succeed with synthetic `resolv.conf` (run a `crunch.fetchurl` build with a network URL) ✅ 4s

## Deferred (documented but out of scope)

- Seccomp syscall filtering (separate change with its own compatibility surface)
- Time namespace (`--unshare-time`, kernel 5.6+, bwrap lacks support)
- Full `/proc` virtualization via FUSE-based procfs filter
