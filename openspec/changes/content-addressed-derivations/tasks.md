## Phase 1: Nickel contract and CrunchDerivation type

- [ ] Add `addressing_mode` field to `lib/derivation.ncl` with enum `[| 'input-addressed, 'content-addressed |]`, default `'content-addressed`
- [ ] Add `addressing_mode` to `CrunchDerivation` struct in `crates/crunch-glue/src/types.rs`, with serde default
- [ ] Add `addressing_mode_to_string` helper in `lib/helpers.ncl`
- [ ] Nickel tests: default is content-addressed, explicit input-addressed works, invalid variant rejected

## Phase 2: Provisional paths and convert() branching

- [ ] In `convert()`, branch on `addressing_mode`: input-addressed uses existing code path, content-addressed sets output paths to `None` and environment to `hash_placeholder(output_name)`
- [ ] Extend `KnownPaths` to track output resolution state: `Option<StorePath>` per output, with `resolve_output()` method
- [ ] Add `get_output_path(drv_path, output_name) -> Option<StorePath>` to KnownPaths
- [ ] Tests: CA derivation from `convert()` has `None` output paths, input-addressed has `Some` (existing behavior)

## Phase 3: Self-reference rewriting primitives

- [ ] Implement `replace_provisional_with_marker(output_bytes: &[u8], provisional: &str) -> (Vec<u8>, bool)` — returns rewritten bytes and whether any self-refs were found. Marker is `\0` repeated to store path length.
- [ ] Implement `replace_marker_with_final(output_bytes: &[u8], final_path: &str) -> Vec<u8>` — replaces zero markers with the final CA path
- [ ] Implement `replace_input_provisional(output_bytes: &[u8], old_path: &str, new_path: &str) -> Vec<u8>` — rewrites input provisional paths to their resolved CA paths
- [ ] Tests: round-trip provisional → marker → final produces correct bytes; no-op when no references present; multiple occurrences all replaced; binary data (non-UTF8) handled

## Phase 4: Post-build CA resolution in Builder

- [ ] After `do_build` returns for a CA derivation: scan output for self-references to provisional path, replace with zero marker
- [ ] Compute NAR hash of marker-replaced output (BLAKE3)
- [ ] Compute final CA store path via `build_ca_path` with `CAHash::Nar`
- [ ] Replace zero markers with final CA path in the output
- [ ] Also replace any input provisional → final CA paths (for transitive CA deps)
- [ ] Move/persist output at the final CA path in the store
- [ ] Call `known_paths.resolve_output()` with the final path
- [ ] Record `has_self_references: bool` in PathInfo for later verification
- [ ] Tests with mock BuildService: CA derivation gets content-based path; two derivations with identical mock output get identical paths; two with different output get different paths

## Phase 5: Multi-output and cache

- [ ] Handle multi-output CA derivations: each output resolved independently
- [ ] CA cache lookup: if PathInfoService has a record mapping this derivation to a CA path and that path exists on disk, skip build
- [ ] Fallback without persistent PathInfoService: always rebuild CA derivations (acceptable degraded mode)
- [ ] Tests: multi-output CA, cache hit with persistent PathInfo, cache miss without

## Phase 6: Integration

- [ ] Integration test: `crunch build` with a CA derivation, verify output path is content-based
- [ ] Integration test: rebuild with identical output produces same path
- [ ] Integration test: mixed graph — input-addressed seed → CA derivation → CA consumer
- [ ] Update README and examples to show `addressing_mode`
