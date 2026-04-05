## Phase 1: Nickel contract and CrunchDerivation type

- [x] Add `addressing_mode` field to `lib/derivation.ncl` with enum `[| 'input-addressed, 'content-addressed |]`, default `'content-addressed`
- [x] Add `addressing_mode` to `CrunchDerivation` struct in `crates/crunch-glue/src/types.rs`, with serde default
- [x] Add `addressing_mode_to_string` helper in `lib/helpers.ncl`
- [x] Nickel tests: default is content-addressed (`ca_json_default_is_content_addressed`), explicit input-addressed works, invalid variant rejected by Nickel contract

## Phase 2: Provisional paths and convert() branching

- [x] In `convert()`, branch on `addressing_mode`: input-addressed uses existing code path, content-addressed sets output paths to `None` and environment to `hash_placeholder(output_name)`
- [x] Extend `KnownPaths` to track output resolution state: `Option<StorePath>` per output, with `resolve_output()` method
- [x] Add `get_output_path(drv_path, output_name) -> Option<StorePath>` to KnownPaths
- [x] Tests: CA derivation from `convert()` has `None` output paths, input-addressed has `Some` (existing behavior)

## Phase 3: Self-reference rewriting primitives

- [x] Implement `replace_provisional_with_marker(output_bytes: &[u8], provisional: &str) -> (Vec<u8>, bool)` — returns rewritten bytes and whether any self-refs were found. Marker is `\0` repeated to store path length.
- [x] Implement `replace_marker_with_final(output_bytes: &[u8], final_path: &str) -> Vec<u8>` — replaces zero markers with the final CA path
- [x] Implement `replace_input_provisional(output_bytes: &[u8], old_path: &str, new_path: &str) -> Vec<u8>` — rewrites input provisional paths to their resolved CA paths
- [x] Tests: round-trip provisional → marker → final produces correct bytes; no-op when no references present; multiple occurrences all replaced; binary data (non-UTF8) handled

## Phase 4: Post-build CA resolution in Builder

- [x] After `do_build` returns for a CA derivation: compute NAR hash, derive CA store path via `build_ca_path_with_store_dir` with `CAHash::Nar(Sha256(nar_sha256))`
- [x] Compute final CA store path and register in PathInfo with `ca` field set
- [x] Call `known_paths.resolve_output()` with the final path
- [x] `build()` signature changed to `&mut KnownPaths` for resolve_output
- [ ] Self-reference rewriting: scan output bytes for provisional placeholder, replace with zero marker, hash, replace with final path (rewrite primitives exist, integration into Builder deferred — requires byte-level access to build output which `BuildService` returns as a `Node`, not raw bytes)
- [ ] Input provisional → final CA path rewriting for transitive CA deps (same constraint)
- [ ] Tests with mock BuildService: CA derivation gets content-based path; two derivations with identical mock output get identical paths

## Phase 5: Multi-output and cache

- [x] Handle multi-output CA derivations: each output resolved independently (loop iterates all outputs)
- [ ] CA cache lookup: needs drv-identity → CA-path mapping (requires schema extension in PathInfoService or separate mapping table)
- [ ] Tests: multi-output CA, cache hit with persistent PathInfo

## Phase 6: Integration

- [ ] Integration test: `crunch build` with a CA derivation, verify output path is content-based
- [ ] Integration test: rebuild with identical output produces same path
- [ ] Integration test: mixed graph — input-addressed seed → CA derivation → CA consumer
- [ ] Update README and examples to show `addressing_mode`
