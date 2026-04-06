## Phase 1: Define DerivationRegistry

- [x] Create `DerivationRegistry` struct in crunch-build with `RegistryEntry` (drv_path, hdm, derivation, content_addressed, resolved_outputs) -- 5m
- [x] Implement `get()`, `get_by_drv_path()`, `insert()`, `resolve_output()`, `store_dir()` -- (in registry.rs)
- [x] Implement `get_hdm_by_drv_path()` for downstream HDM lookups -- (in registry.rs)
- [x] Add unit tests for DerivationRegistry (insert, lookup, resolve, dedup) -- 12 tests

## Phase 2: Rename KnownPaths -> ConversionCache

- [x] Rename `KnownPaths` to `ConversionCache` in crunch-glue -- known_paths.rs -> conversion_cache.rs
- [x] Remove `resolve_output()`, `content_addressed`, `resolved_outputs` from ConversionCache (build-time concerns) -- removed resolved_outputs, resolve_output, get_output_path
- [x] Keep `begin_conversion()`, `end_conversion()`, `insert()`, `get_hdm_by_drv_path()`, `store_dir()`
- [x] Add `iter_entries()` method that yields (drv_path, hdm, derivation, content_addressed) for registry population
- [x] Update crunch-glue tests

## Phase 3: Bridge function

- [x] Write `populate_registry(cache: &ConversionCache, registry: &mut DerivationRegistry)` in crunch-build/registry.rs
- [x] Wire it into the build pipeline: convert -> populate -> build (main.rs + bootstrap.rs)

## Phase 4: Update crunch-build

- [x] Replace all `KnownPaths` imports with `DerivationRegistry` in worker.rs, orchestrate.rs, build_request.rs, dynamic.rs
- [x] Remove `crunch-glue` from crunch-build/Cargo.toml `[dependencies]` (kept as `[dev-dependencies]` for integration tests)
- [x] Verify `cargo build -p crunch-build` compiles without crunch-glue in production deps
- [x] Update all crunch-build tests to use DerivationRegistry
- [x] Update integration tests (integration_build.rs, stdlib_tests.rs) to use bridge pattern
- [x] All 131 workspace tests pass
