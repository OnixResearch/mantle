## Phase 1: nix-compat store_dir plumbing

- [ ] Add `build_text_path_with_store_dir`, `build_ca_path_with_store_dir`, `build_output_path_with_store_dir` to `vendor/nix-compat/src/store_path/utils.rs` — delegate to `build_store_path_from_fingerprint_parts_with_store_dir`
- [ ] Add `calculate_derivation_path_with_store_dir` to `vendor/nix-compat/src/derivation/mod.rs`
- [ ] Add `calculate_output_paths_with_store_dir` to `vendor/nix-compat/src/derivation/mod.rs`
- [ ] Tests: same derivation with different store_dir produces different paths; default store_dir matches existing paths

## Phase 2: crunch-glue threading

- [ ] Change `KnownPaths::new()` to `KnownPaths::new(store_dir: String)`, store the prefix, use it in `insert()` and `get_by_drv_path()` key serialization
- [ ] Add `store_dir: &str` parameter to `convert()` and `convert_inner()`, pass to `calculate_output_paths_with_store_dir` and `calculate_derivation_path_with_store_dir`
- [ ] Update `to_absolute_path()` calls in convert.rs to `to_absolute_path_with_prefix(store_dir)`
- [ ] Update all crunch-glue tests — `KnownPaths::new()` calls gain the store_dir arg, path assertions remain `/nix/store` (default)
- [ ] Add test: `convert()` with `store_dir = "/opt/crunch"` produces paths starting with `/opt/crunch/`

## Phase 3: crunch-build threading

- [ ] Add `store_dir: &str` parameter to `derivation_to_build_request()`, set `NIX_STORE` and `inputs_dir` from it
- [ ] Change `Builder` to store `store_dir: String`, use `to_absolute_path_with_prefix` in `all_outputs_exist`, `path_exists_on_disk`, `load_cached_outputs`, `ensure_input_nodes`
- [ ] Replace all `to_absolute_path()` in orchestrate.rs with `to_absolute_path_with_prefix(&self.store_dir)`
- [ ] Replace `to_absolute_path()` in build_request.rs placeholder replacement and refscan needle construction
- [ ] Rewrite `builder_skips_build_when_output_exists` test to use a tempdir as store_dir
- [ ] Update remaining crunch-build tests — `Builder::new()` calls pass store_dir, mock tests use `/nonexistent-store` as before (no filesystem checks in those)

## Phase 4: CLI wiring

- [ ] Pass `args.store` to `KnownPaths::new()`, `convert()`, and `Builder::new()` in main.rs `cmd_build()`
- [ ] Use `to_absolute_path_with_prefix` for output path printing in `cmd_build()`
- [ ] Integration test: `crunch build --store <tempdir> hello.ncl` succeeds and prints paths with the custom prefix
