## Phase 1: Store library — NAR rendering and push core

- [x] Promote `render_nar_bytes` pattern to `StoreHandle::render_nar<W: AsyncWrite>(&self, node: &Node, dest: &mut W)` public method in `crates/crunch-store/src/handle.rs`
- [x] Add unit test: `render_nar` output sha256 matches PathInfo.nar_sha256 for a blob node (covered by `push_single_signed_path`)
- [x] Create `crates/crunch-store/src/push.rs` with `PushReport`, `PushedPath`, and `PushOptions` structs
- [x] Implement `export_paths_to_cache_dir(handle, paths, dest, options) -> Result<PushReport>` in push module
- [x] Write `nix-cache-info` generation: `StoreDir` from `handle.store_dir()`, `WantMassQuery: 1`, `Priority: 30`
- [x] Add unit tests: single signed path push, unsigned skip, idempotent skip, multi-path push, narinfo round-trip parse, nix-cache-info preservation, references match

## Phase 2: CLI subcommand

- [x] Add `Push` variant to `StoreAction` enum in `src/main.rs` with `--to`, `--all`, `--trust-unsigned`, and positional path args
- [x] Implement `cmd_store_push()` in `src/store_cmd.rs`: resolve selectors, acquire store mutation lock, call `export_paths_to_cache_dir`, print summary
- [x] Add path selector resolution: match positional args against PathInfo store paths (full or fragment match, same pattern as `crunch store info`)
- [ ] Add integration test: build a hello derivation, push it, verify narinfo parses and NAR sha256 matches

## Phase 3: Store prefix and cross-compat validation

- [x] Verify `NarInfo::Display` renders the correct `StorePath` for non-`/nix/store` prefixes — patch or document if needed
- [x] Add test: push under `/crunch/store` prefix produces narinfo with `StorePath: /crunch/store/...`
- [ ] Add test: push under `/nix/store` prefix produces narinfo consumable by `nix-store --verify-path` (if host nix available)
