## Phase 1: KnownPaths unit tests

- [x] Test insert + get_by_aterm_hash round-trip ✅ (pre-existing in known_paths.rs)
- [x] Test get_hdm_by_drv_path returns correct HDM ✅ (pre-existing in known_paths.rs)
- [x] Test get_by_drv_path returns correct entry ✅ (pre-existing in known_paths.rs)
- [x] Test get_by_drv_path returns None for unknown path ✅ (pre-existing in known_paths.rs)
- [x] Test cycle detection: begin_conversion returns false on second call ✅ (pre-existing in known_paths.rs)
- [x] Test end_conversion allows re-entry ✅ (pre-existing in known_paths.rs)

## Phase 2: convert() — basic derivations

- [x] Test simple derivation: one output, no inputs, verify exact store path ✅ simple_drv_exact_store_path
- [x] Test builder and system propagate to nix_compat::Derivation fields ✅ builder_and_system_propagate
- [x] Test environment includes system, builder, name, and output path ✅ environment_auto_populated
- [x] Test multi-output derivation: outputs list reflected in Derivation.outputs ✅ multi_output_reflected_in_derivation
- [x] Test each output gets a distinct computed path ✅ multi_output_distinct_paths

## Phase 3: convert() — inputs and dependencies

- [x] Test source input: Input::Source wires to input_sources ✅ source_input_wires_to_input_sources
- [x] Test derivation input: Input::Derivation wires to input_derivations ✅ derivation_input_wires_to_input_derivations
- [x] Test nested derivation: inner drv registered in KnownPaths ✅ nested_drv_registered_in_known_paths
- [x] Test diamond dependency: two inputs sharing a dep produce one KnownPaths entry ✅ diamond_dep_single_known_paths_entry
- [x] Test cycle detection: self-referencing input returns CircularDependency error ✅ cycle_returns_circular_dependency_error

## Phase 4: convert() — fixed-output derivations

- [x] Test FOD flat sha256: ca_hash set on output, correct mode ✅ fod_flat_sha256
- [x] Test FOD recursive sha256: ca_hash is Nar variant ✅ fod_recursive_sha256
- [x] Test FOD with SRI hash format parses correctly ✅ fod_sri_hash_parses + fod_sri_parses_to_flat_ca_hash
- [x] Test FOD with hex hash format parses correctly ✅ fod_hex_parses_to_flat_ca_hash
- [x] Test invalid hash algo returns error ✅ fod_invalid_algo_rejected
- [x] Test invalid hash mode returns error ✅ fod_invalid_mode_rejected
