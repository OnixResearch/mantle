## Phase 1: Bootstrap .ncl portability

- [x] Replace `/nix/store` globs with `$NIX_STORE` in `bootstrap/make.ncl`
- [x] Replace `/nix/store` globs with `$NIX_STORE` in `bootstrap/dash.ncl`
- [x] Replace `/nix/store` globs with `$NIX_STORE` in `bootstrap/binutils.ncl`
- [x] Replace `/nix/store` globs with `$NIX_STORE` in `bootstrap/musl.ncl`
- [x] Replace `/nix/store` globs with `$NIX_STORE` in `bootstrap/gcc.ncl`
- [x] Replace `/nix/store` globs with `$NIX_STORE` in `bootstrap/busybox.ncl`
- [x] Replace `/nix/store` globs with `$NIX_STORE` in `bootstrap/bwrap.ncl`
- [x] Replace `/nix/store` globs with `$NIX_STORE` in `bootstrap/rust.ncl`
- [x] Replace `/nix/store` globs with `$NIX_STORE` in `bootstrap/crunch.ncl`
- [x] Replace `/nix/store` globs with `$NIX_STORE` in `bootstrap/selftest.ncl`
- [x] Replace `/nix/store` globs with `$NIX_STORE` in `bootstrap/integration-test.ncl`
- [x] Replace `/nix/store` globs with `$NIX_STORE` in `self_build.rs::generate_self_build_ncl()`

## Phase 2: Smoke test isolation

- [ ] Add `--state-dir` CLI flag or `CRUNCH_STATE_DIR` env var
- [ ] Update `build_ncl` helper in `tests/smoke.rs` to use per-test state dir
- [ ] Verify `smoke_build_cached_on_second_run` passes reliably

## Phase 3: Multi-derivation eval

- [ ] Decide: support arrays in eval entry point OR update test to use record syntax
- [ ] Implement the chosen approach
- [ ] Verify `smoke_build_multi_derivation_file` passes
