## Phase 1: Pure function tests in build_request.rs

- [ ] Test `replace_placeholders` substitutes output path for hash_placeholder
- [ ] Test `replace_placeholders` no-op when string has no placeholders
- [ ] Test `replace_placeholders` handles multi-output (two different placeholders)
- [ ] Test `replace_placeholders_bstr` matches String variant behavior
- [ ] Test `collect_input_paths` with source-only derivation
- [ ] Test `collect_input_paths` with derivation-only inputs (needs KnownPaths)
- [ ] Test `collect_input_paths` with mixed sources and derivation inputs
- [ ] Test `collect_input_paths` returns DerivationNotFound for missing input drv

## Phase 2: Pure function tests in orchestrate.rs

- [ ] Test `resolve_references` maps needle index 0 to first output path
- [ ] Test `resolve_references` maps indices past outputs to input paths
- [ ] Test `resolve_references` ignores out-of-range indices
- [ ] Test `resolve_references` with empty found_needles returns empty vec

## Phase 3: MockBuildService

- [ ] Implement `MockBuildService` that records calls and returns canned results
- [ ] Test Builder with a single no-input derivation calls do_build once
- [ ] Test Builder with a chain (A → B) builds A before B
- [ ] Test Builder with diamond (A → B, A → C, B+C → D) builds A once

## Phase 4: Cache behavior

- [ ] Test `all_outputs_exist` returns false when output path missing
- [ ] Test `all_outputs_exist` returns true when output path exists (tempdir)
- [ ] Test Builder skips do_build when all outputs exist on disk
- [ ] Test Builder returns `cached: true` in BuildOutcome for cached builds
