# Tasks: Overlay Store Composition

## Phase 1: Vendored read-through primitive

- [ ] [parallel] Add a read-only-far / no-backfill mode to the vendored `Cache` DirectoryService combinator (`vendor/snix-castore/src/directoryservice/combinators.rs`): base hits return without `put` into near. Default behavior unchanged. r[store_transports.overlay_composition]
- [ ] [parallel] Add the no-backfill read mode to `CombinedBlobService` (`vendor/snix-castore/src/blobservice/combinator.rs`). r[store_transports.overlay_composition]
- [ ] [parallel] Add the no-backfill read mode to the PathInfoService `Cache` combinator (`vendor/snix-store/src/pathinfoservice/cache.rs`). r[store_transports.overlay_composition]

## Phase 2: crunch-store overlay composition

- [ ] [serial] Add `StoreConfig.base_state_dirs: Vec<PathBuf>` and `StoreHandle::open_overlay` that wires overlay services over read-only base services using the no-backfill combinators. Enforce the same-prefix invariant. r[store_transports.overlay_composition]
- [ ] [serial] Route all writes (`put`, `put_multiple_start`, pathinfo `put`, blob writes, `persist_and_export_signed_output`) to the overlay only; open bases read-only and reject base writes. r[store_transports.overlay_composition]
- [ ] [serial] Add the `StoreLayer { Overlay, Base }` provenance tag to consumed nodes/PathInfo and thread it through attestation synthesis. r[store_transports.overlay_provenance_layer]
- [ ] [serial] Extend overlay GC reachability to include base-referenced paths so an overlay GC cannot dangle a closure that reads through the base; never mutate the base. r[store_transports.overlay_gc_cross_layer]

## Phase 3: CLI and pipeline wiring

- [ ] [serial] Add the repeatable ordered `--base-store <state-dir>` global CLI option and fail closed on missing/unopenable bases. r[store_transports.overlay_cli_declaration]
- [ ] [serial] Route `resolve_and_ingest_sources` / `cached_node_for_path` in `crates/crunch-build/src/orchestrate.rs` through the composed handle so the sandbox sees the merged castore view. r[store_transports.overlay_composition]

## Phase 4: Tests

- [ ] [parallel] Positive test: base hit returns value without mutating the overlay blob/directory/PathInfo stores. r[store_transports.overlay_composition.scenario.read-through-no-backfill]
- [ ] [parallel] Positive test: overlay PathInfo shadows base PathInfo for the same store path. r[store_transports.overlay_composition.scenario.shadow]
- [ ] [parallel] Positive test: writes land in overlay only; base is unmodified. r[store_transports.overlay_composition.scenario.write-routing]
- [ ] [parallel] Negative test: prefix mismatch between overlay and base is a hard error naming both prefixes. r[store_transports.overlay_composition.scenario.prefix-mismatch]
- [ ] [parallel] Positive test: base-sourced path attestation records base as producing layer and applies base trust. r[store_transports.overlay_provenance_layer.scenario.base-trust]
- [ ] [parallel] Negative test: shadowed overlay path with unsigned PathInfo is treated as unsigned and does not inherit base signature trust. r[store_transports.overlay_provenance_layer.scenario.no-inherited-trust]
- [ ] [parallel] Positive test: overlay GC marks base-referenced paths live and does not remove base content. r[store_transports.overlay_gc_cross_layer.scenario.base-ref-survives]
- [ ] [parallel] Positive test: two `--base-store` declarations consult base A before base B. r[store_transports.overlay_cli_declaration.scenario.ordered-stack]
- [ ] [parallel] Negative test: missing `--base-store` state directory fails closed before any build or store operation. r[store_transports.overlay_cli_declaration.scenario.missing-base-fails]

## Phase 5: Verification and archive

- [ ] [serial] Run `cargo test -p crunch-store` and the overlay scenario tests with isolated `CARGO_TARGET_DIR`. r[store_transports.overlay_composition] r[store_transports.overlay_provenance_layer] r[store_transports.overlay_gc_cross_layer]
- [ ] [serial] Run `cairn validate --root .` and `cairn gate proposal|design|tasks overlay-store-composition --root .`. r[store_transports.overlay_composition]
- [ ] [serial] Sync this delta into accepted `store-transports` specs, archive the package, and commit the verified slice. r[store_transports.overlay_composition]
