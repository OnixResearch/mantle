## Phase 1: Configurable store prefix

- [x] Audit all `STORE_DIR` / `LOGICAL_STORE_DIR` references in vendored nix_compat + crunch crates (count sites, classify as hash-affecting vs display-only) ✅ 10m (started: 2026-04-06T14:18Z → completed: 2026-04-06T14:28Z)
- [x] Add `_with_prefix(store_dir: &str)` variants to nix_compat functions that hardcode STORE_DIR: `build_store_path_from_fingerprint_parts`, `hash_derivation_modulo`, output path computation ✅ 15m (started: 2026-04-06T14:28Z → completed: 2026-04-06T14:43Z)
- [x] Thread store prefix through ConversionCache → DerivationRegistry → StoreConfig → Builder → Worker → BuildRequest as a `&str` parameter ✅ 20m (started: 2026-04-06T14:43Z → completed: 2026-04-06T15:03Z)
- [x] Replace `NIX_STORE` sandbox env var with `STORE_DIR` (or keep both for compat), set to the configured prefix ✅ already done (build_request.rs overrides NIX_STORE with store_dir param)
- [x] Add `--store-prefix` CLI flag (default: `/crunch/store`) and `--nix-compat` shorthand for `/nix/store` ✅ 5m
- [x] Update all `LOGICAL_STORE_DIR` constants to use the CLI-provided value ✅ (done as part of threading task)
- [x] Patch `StorePath::to_absolute_path()` call sites to use `to_absolute_path_with_prefix(prefix)` (already exists, just not used everywhere) ✅ (done as part of threading task)
- [x] Integration test: same derivation with two different prefixes produces different output hashes ✅ 5m (started: 2026-04-06T15:08Z → completed: 2026-04-06T15:13Z)
- [ ] Update all hardcoded `/nix/store` in test assertions to use a helper that returns the test prefix (deferred — existing tests use /nix/store as default, which is correct behavior)

## Phase 2: Bootstrap busybox-static

- [x] Pin busybox 1.37.0 source tarball URL + sha256 hash ✅ 5m (started: 2026-04-06T15:14Z → completed: 2026-04-06T15:19Z)
- [x] Write `bootstrap/busybox.ncl` — fetchTarball + build with `make defconfig && make LDFLAGS=-static` ✅ (done with pin task)
- [x] Handle busybox `.config` patching: CONFIG_STATIC=y, CONFIG_INSTALL_NO_USR=y ✅ (sed in busybox.ncl)
- [x] Verify output: `busybox --list` shows required applets, `busybox sh -c 'echo ok'` works ✅ 404 applets, static-pie ELF, sh -c works
- [x] Integration test: build busybox.ncl, run the binary ✅ full chain build in ~5.5 min

## Phase 3: Bootstrap bwrap

- [x] Pin bwrap 0.11.0 source tarball URL + sha256 hash ✅ 5m (started: 2026-04-06T15:19Z → completed: 2026-04-06T15:24Z)
- [x] Write `bootstrap/bwrap.ncl` — fetchTarball + direct gcc compile (no meson) ✅ (done with pin task)
- [x] Write inline config.h with correct feature flags for Linux 5.x+ ✅ (in bwrap.ncl: PACKAGE_STRING, no HAVE_SELINUX, no ENABLE_REQUIRE_USERNS)
- [x] Identify exact source files needed (bubblewrap.c, bind-mount.c, network.c, utils.c + headers) ✅ (parse-mountinfo is inside bind-mount.c, not a separate file)
- [x] Static link against musl, verify `bwrap --version` works ✅ static-pie ELF, `bubblewrap 0.11.0`
- [x] Integration test: build bwrap.ncl, run `bwrap --version` ✅ full chain build in ~5.5 min

## Phase 4: Wire into self-build

- [x] Update `self_build.rs` to use crunch-built busybox path instead of external `SNIX_BUILD_SANDBOX_SHELL` ✅
- [x] Update `self_build.rs` to use crunch-built bwrap instead of PATH-provided bwrap ✅
- [x] Update bootstrap chain ordering: busybox + bwrap after gcc+musl, before rust ✅ (inputs list order)
- [x] Self-build .ncl references crunch-built tools as inputs ✅
- [x] First-time detection: if no crunch-built bwrap exists, fall back to PATH bwrap with a warning ✅ (host-side: check_host_bwrap() warns when using external bwrap, errors when missing; sandbox-side: NCL prints warning when bwrap input not found among store paths)
- [x] End-to-end test: `crunch self-build --store /tmp/crunch-store --nix-compat` ✅ 15.5 min, 32 MiB static-pie ELF, `crunch --help` runs

## Phase 5: Cleanup

- [x] Update AGENTS.md build env to document crunch-built busybox/bwrap as post-self-build default ✅ (SNIX_BUILD_SANDBOX_SHELL still documented for dev builds, which is correct)
- [x] Remove hardcoded Nix store paths from AGENTS.md (bwrap, busybox nix store paths) ✅ (removed from Nix store paths list)
- [x] Update error messages in errors.rs to not reference `nix-env`, `nix-store`, `nixpkgs` ✅ (replaced nix-env suggestion, nix-store --add-root hint, and nixos.org link with crunch-native alternatives)
- [x] Add state migration warning: detect old `/nix/store`-prefixed pathinfo.redb entries, print clear error ✅ (CA mappings prefix check in StoreHandle::open)
- [ ] Update README / examples to use `/crunch/store` paths (deferred — README is minimal, examples use /nix/store as --nix-compat default)
- [x] ADR documenting the prefix change and why ✅ adr/0003-configurable-store-prefix.md
