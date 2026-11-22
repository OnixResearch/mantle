# Change: Verify each hydrated input once per realization

## Why

`foreign-import realize` on route `cache-inputs-build-roots-v2` re-reads the whole hydrated input
closure after hydration. In the Octet pilot (21.7 GB closure) the process read 72 GiB and spent
about 19 minutes before the sandbox started; an unchanged reuse run spent about 4 minutes for no
work at all.

Three reads come from two places:

- `crunch_store::import_http_cache_closure_with_validator` already exports every closure member to
  the physical store and checks it against its signed NAR in this run
  (`export_admitted_path`: staged export plus `materialized_nar_matches`; present members through
  `ensure_local_member_exported`).
- `src/foreign_realization_shell.rs::hydrate_signed_cache_closure` then calls
  `StoreHandle::export_cached_path_info` for every member, whose
  `export_output_if_needed(.., is_root = true)` re-ingests each existing export through
  `exported_path_matches_node` (`ingest_path` over the whole tree).
- `locally_present_input_paths`, run after hydration on route v2 and again on every reuse run,
  calls `export_cached_path_info` for every output of every non-root unit: the only full re-ingest
  on a reuse run.

## What Changes

- Add a lookup-only `StoreHandle::cached_path_info(&mut self, path)` that returns the complete local
  `PathInfo` with the same checks `export_cached_path_info` applies before its export step, and
  neither materializes the path nor records a built output.
- Refactor `export_cached_path_info` onto `cached_path_info` so the two cannot drift.
- Use `cached_path_info` in `hydrate_signed_cache_closure`'s member walk and in
  `locally_present_input_paths`. Keep `export_cached_path_info` for selected roots, the cache-only
  path, and route v1, which still rely on physical export.
- State the consequence: a reuse run no longer re-verifies or repairs physical exports of inputs.
  Builds read inputs from castore, and the selected root keeps its own NAR check in
  `classify_present_root`.

## Impact

- **Files**: `crates/crunch-store/src/handle.rs`, `src/foreign_realization_shell.rs`.
- **Testing**: `cargo test -p crunch-store --lib` (new `cached_path_info` unit tests),
  `cargo test -p mantle --test cache_inputs_build_route --test foreign_import_cli` unchanged,
  `cargo test -p mantle --bin mantle -- foreign_`, fmt and strict Clippy for `-p mantle`,
  `-p crunch-store`.
- **Non-goals**: changing the closure import's own export/verification, changing receipt fields or
  bytes, changing selected-root classification, or changing route v1 or the cache-only route.
