## Phase 1: Store library — NAR rendering and push core

- [ ] Promote `render_nar_bytes` pattern to `StoreHandle::render_nar<W: AsyncWrite>(&self, node: &Node, writer: W)` public method in `crates/crunch-store/src/handle.rs`
- [ ] Add unit test: `render_nar` output sha256 matches PathInfo.nar_sha256 for a blob node and a directory node
- [ ] Create `crates/crunch-store/src/push.rs` with `PushReport`, `PushedPath`, and `PushOptions` structs
- [ ] Implement `export_paths_to_cache_dir(handle, paths, dest, options) -> Result<PushReport>` in push module
- [ ] Write `nix-cache-info` generation: `StoreDir` from `handle.store_dir()`, `WantMassQuery: 1`, `Priority: 30`
- [ ] Add unit tests: single signed path push, unsigned skip, idempotent skip, multi-path push, narinfo round-trip parse

## Phase 2: CLI subcommand

- [ ] Add `Push` variant to `StoreAction` enum in `src/main.rs` with `--to`, `--all`, `--trust-unsigned`, `--signing-key`, and positional path args
- [ ] Implement `cmd_store_push()` in `src/store_cmd.rs`: resolve selectors, acquire store mutation lock, call `export_paths_to_cache_dir`, print summary
- [ ] Add path selector resolution: match positional args against PathInfo store paths (full or fragment match, same pattern as `crunch store info`)
- [ ] Add integration test: build a hello derivation, push it, verify narinfo parses and NAR sha256 matches

## Phase 3: Store prefix and cross-compat validation

- [ ] Verify `NarInfo::Display` renders the correct `StorePath` for non-`/nix/store` prefixes — patch or document if needed
- [ ] Add test: push under `/crunch/store` prefix produces narinfo with `StorePath: /crunch/store/...`
- [ ] Add test: push under `/nix/store` prefix produces narinfo consumable by `nix-store --verify-path` (if host nix available)
