## Phase 1: Pure profile model

- [ ] [serial] Add the built-in Cargo profile table (`dev`, `release`, `test` inherits `dev`, `bench` inherits `release`) as pure functions with no I/O. r[rust_package_planning.profile_defaults_table]
- [ ] [parallel] Add positive resolution tests for all four built-in profiles and negative tests for unknown and empty profile names. r[rust_package_planning.profile_defaults_table]

## Phase 2: Codegen flag lowering

- [ ] [serial] Lower `opt-level`, `debuginfo`, `debug-assertions`, `overflow-checks`, and `codegen-units` into the native target and host unit rustc argument builders. r[rust_package_planning.profile_codegen_flags]
- [ ] [serial] Lower the same flags into the Cargo-derived unit, integration-test, and dev-dependency lib argument builders. r[rust_package_planning.profile_codegen_flags]
- [ ] [parallel] Add positive tests that dev and release produce different codegen arguments and negative tests that no builder emits these flags from ambient environment state. r[rust_package_planning.profile_codegen_flags]

## Phase 3: Identity and determinism

- [ ] [serial] Add resolved profile material to `rustc_unit_metadata_disambiguator` and assert dev/release disambiguators differ for identical crate facts. r[rust_package_planning.profile_unit_identity]
- [ ] [serial] Record explicit `incremental = false` and the selected `codegen-units` values in unit receipts. r[rust_package_planning.profile_determinism_policy]
- [ ] [parallel] Add negative tests that unsupported `lto`, `panic`, `rpath`, `strip`, and `split-debuginfo` requests fail closed with a deterministic blocker. r[rust_package_planning.profile_determinism_policy]

## Phase 4: Verification

- [ ] [serial] Run focused `cargo test` for the changed planning surfaces, then rerun the affected topology self-probe and record any changed artifact identities as intended. r[rust_package_planning.profile_codegen_flags]
