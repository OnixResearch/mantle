## Phase 1: Pure function tests in build_request.rs

- [x] Test `replace_placeholders` substitutes output path for hash_placeholder ✅ replace_placeholders_substitutes_output_path
- [x] Test `replace_placeholders` no-op when string has no placeholders ✅ replace_placeholders_noop_without_placeholder
- [x] Test `replace_placeholders` handles multi-output (two different placeholders) ✅ replace_placeholders_multi_output
- [x] Test `replace_placeholders_bstr` matches String variant behavior ✅ replace_placeholders_bstr_matches_string_variant
- [x] Test `collect_input_paths` with source-only derivation ✅ collect_inputs_source_only
- [x] Test `collect_input_paths` with derivation-only inputs (needs KnownPaths) ✅ collect_inputs_derivation_only
- [x] Test `collect_input_paths` with mixed sources and derivation inputs ✅ collect_inputs_mixed
- [x] Test `collect_input_paths` returns DerivationNotFound for missing input drv ✅ collect_inputs_missing_drv_returns_error

## Phase 2: Pure function tests in orchestrate.rs

- [x] Test `resolve_references` maps needle index 0 to first output path ✅ resolve_refs_index_zero_maps_to_output
- [x] Test `resolve_references` maps indices past outputs to input paths ✅ resolve_refs_index_past_outputs_maps_to_input
- [x] Test `resolve_references` ignores out-of-range indices ✅ resolve_refs_out_of_range_ignored
- [x] Test `resolve_references` with empty found_needles returns empty vec ✅ resolve_refs_empty_needles

## Phase 3: MockBuildService

- [x] Implement `MockBuildService` that records calls and returns canned results ✅ shared blob service approach
- [x] Test Builder with a single no-input derivation calls do_build once ✅ builder_single_drv_calls_do_build_once
- [x] Test Builder with a chain (A → B) builds A before B ✅ builder_chain_builds_dep_first
- [x] Test Builder with diamond (A → B, A → C, B+C → D) builds A once ✅ builder_diamond_builds_shared_dep_once

## Phase 4: Cache behavior

- [x] Test Builder skips do_build when all outputs exist on disk ✅ builder_skips_build_when_output_exists (skips gracefully if /nix/store is read-only)
- [x] Test Builder returns `cached: true` in BuildOutcome for cached builds ✅ same test
