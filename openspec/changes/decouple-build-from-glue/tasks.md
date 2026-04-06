## Phase 1: Define DerivationRegistry

- [ ] Create `DerivationRegistry` struct in crunch-build with `RegistryEntry` (drv_path, hdm, derivation, content_addressed, resolved_outputs)
- [ ] Implement `get()`, `get_by_drv_path()`, `insert()`, `resolve_output()`, `store_dir()`
- [ ] Implement `get_hdm_by_drv_path()` for downstream HDM lookups
- [ ] Add unit tests for DerivationRegistry (insert, lookup, resolve, dedup)

## Phase 2: Rename KnownPaths → ConversionCache

- [ ] Rename `KnownPaths` to `ConversionCache` in crunch-glue
- [ ] Remove `resolve_output()`, `content_addressed`, `resolved_outputs` from ConversionCache (build-time concerns)
- [ ] Keep `begin_conversion()`, `end_conversion()`, `insert()`, `get_hdm_by_drv_path()`, `store_dir()`
- [ ] Add `iter_entries()` method that yields (drv_path, hdm, derivation, content_addressed) for registry population
- [ ] Update crunch-glue tests

## Phase 3: Bridge function

- [ ] Write `populate_registry(cache: &ConversionCache, registry: &mut DerivationRegistry)` in crunch-pipeline (or temporarily in main.rs)
- [ ] Wire it into the build pipeline: convert → populate → build

## Phase 4: Update crunch-build

- [ ] Replace all `KnownPaths` imports with `DerivationRegistry` in worker.rs, orchestrate.rs, build_request.rs, dynamic.rs
- [ ] Remove `crunch-glue` from crunch-build/Cargo.toml
- [ ] Verify `cargo build -p crunch-build` compiles without crunch-glue
- [ ] Update all crunch-build tests to use DerivationRegistry
