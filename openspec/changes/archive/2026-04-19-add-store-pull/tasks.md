## Phase 1: Store library — pull core

- [x] Create `crates/crunch-store/src/pull.rs` with `PullReport`, `PulledPath`, `PullOptions` structs
- [x] Implement `import_paths_from_cache_dir(handle, source, paths_filter, options) -> Result<PullReport>` — scan narinfos, verify signatures, ingest NARs, persist PathInfo
- [x] Add narinfo signature verification step using `nix_compat::narinfo::VerifyingKey::verify()` before NAR ingestion
- [x] Add store prefix validation: reject narinfos whose `StorePath` prefix differs from `handle.store_dir()`
- [x] Add `pub mod pull;` and re-exports in `crates/crunch-store/src/lib.rs`
- [x] Add unit tests: push→pull round-trip (single path), already-present skip, untrusted signature skip, NAR hash mismatch detection, missing NAR file skip, store prefix mismatch rejection, path filter, multi-path import

## Phase 2: CLI subcommand

- [x] Add `Pull` variant to `StoreAction` enum in `src/main.rs` with `--from`, `--all`, `--trust-unsigned`, `--trusted-public-keys`, and positional path args
- [x] Implement `cmd_store_pull()` in `src/store_cmd.rs`: validate source dir, acquire store mutation lock, call `import_paths_from_cache_dir`, print summary
- [x] Add integration test: build → push → pull into fresh store → verify PathInfo matches and output exists on disk
