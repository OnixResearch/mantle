## Phase 1: KnownPaths unit tests

- [ ] Test insert + get_by_aterm_hash round-trip
- [ ] Test get_hdm_by_drv_path returns correct HDM
- [ ] Test get_by_drv_path returns correct entry
- [ ] Test get_by_drv_path returns None for unknown path
- [ ] Test cycle detection: begin_conversion returns false on second call
- [ ] Test end_conversion allows re-entry

## Phase 2: convert() — basic derivations

- [ ] Test simple derivation: one output, no inputs, verify exact store path
- [ ] Test builder and system propagate to nix_compat::Derivation fields
- [ ] Test environment includes system, builder, name, and output path
- [ ] Test multi-output derivation: outputs list reflected in Derivation.outputs
- [ ] Test each output gets a distinct computed path

## Phase 3: convert() — inputs and dependencies

- [ ] Test source input: Input::Source wires to input_sources
- [ ] Test derivation input: Input::Derivation wires to input_derivations
- [ ] Test nested derivation: inner drv registered in KnownPaths
- [ ] Test diamond dependency: two inputs sharing a dep produce one KnownPaths entry
- [ ] Test cycle detection: self-referencing input returns CircularDependency error

## Phase 4: convert() — fixed-output derivations

- [ ] Test FOD flat sha256: ca_hash set on output, correct mode
- [ ] Test FOD recursive sha256: ca_hash is Nar variant
- [ ] Test FOD with SRI hash format parses correctly
- [ ] Test FOD with hex hash format parses correctly
- [ ] Test invalid hash algo returns error
- [ ] Test invalid hash mode returns error
