## Phase 1: Configurable store prefix

- [ ] Audit all `STORE_DIR` / `LOGICAL_STORE_DIR` references in vendored nix_compat + crunch crates (count sites, classify as hash-affecting vs display-only)
- [ ] Add `_with_prefix(store_dir: &str)` variants to nix_compat functions that hardcode STORE_DIR: `build_store_path_from_fingerprint_parts`, `hash_derivation_modulo`, output path computation
- [ ] Thread store prefix through ConversionCache → DerivationRegistry → StoreConfig → Builder → Worker → BuildRequest as a `&str` parameter
- [ ] Replace `NIX_STORE` sandbox env var with `STORE_DIR` (or keep both for compat), set to the configured prefix
- [ ] Add `--store-prefix` CLI flag (default: `/crunch/store`) and `--nix-compat` shorthand for `/nix/store`
- [ ] Update all `LOGICAL_STORE_DIR` constants to use the CLI-provided value
- [ ] Patch `StorePath::to_absolute_path()` call sites to use `to_absolute_path_with_prefix(prefix)` (already exists, just not used everywhere)
- [ ] Integration test: same derivation with two different prefixes produces different output hashes
- [ ] Update all hardcoded `/nix/store` in test assertions to use a helper that returns the test prefix

## Phase 2: Bootstrap busybox-static

- [ ] Pin busybox 1.37.0 source tarball URL + sha256 hash
- [ ] Write `bootstrap/busybox.ncl` — fetchTarball + build with `make defconfig && make LDFLAGS=-static`
- [ ] Handle busybox `.config` patching: CONFIG_STATIC=y, CONFIG_INSTALL_NO_USR=y
- [ ] Verify output: `busybox --list` shows required applets, `busybox sh -c 'echo ok'` works
- [ ] Integration test: build busybox.ncl, run the binary

## Phase 3: Bootstrap bwrap

- [ ] Pin bwrap 0.11.0 source tarball URL + sha256 hash
- [ ] Write `bootstrap/bwrap.ncl` — fetchTarball + direct gcc compile (no meson)
- [ ] Write inline config.h with correct feature flags for Linux 5.x+
- [ ] Identify exact source files needed (bubblewrap.c, bind-mount.c, network.c, utils.c, parse-mountinfo.c + headers)
- [ ] Static link against musl, verify `bwrap --version` works
- [ ] Integration test: build bwrap.ncl, run `bwrap --version`

## Phase 4: Wire into self-build

- [ ] Update `self_build.rs` to use crunch-built busybox path instead of external `SNIX_BUILD_SANDBOX_SHELL`
- [ ] Update `self_build.rs` to use crunch-built bwrap instead of PATH-provided bwrap
- [ ] Update bootstrap chain ordering: busybox + bwrap after gcc+musl, before rust
- [ ] Self-build .ncl references crunch-built tools as inputs
- [ ] First-time detection: if no crunch-built bwrap exists, fall back to PATH bwrap with a warning
- [ ] End-to-end test: `crunch self-build --store /tmp/test-store` with `/crunch/store` prefix, verify output binary runs

## Phase 5: Cleanup

- [ ] Remove `SNIX_BUILD_SANDBOX_SHELL` from AGENTS.md build env (replaced by crunch-built path)
- [ ] Remove hardcoded Nix store paths from AGENTS.md (bwrap, busybox nix store paths)
- [ ] Update error messages in errors.rs to not reference `nix-env`, `nix-store`, `nixpkgs`
- [ ] Add state migration warning: detect old `/nix/store`-prefixed pathinfo.redb entries, print clear error
- [ ] Update README / examples to use `/crunch/store` paths
- [ ] ADR documenting the prefix change and why
