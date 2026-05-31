# I4 graph validation

Task-ID: I4
Covers: build.engine.dynamic.plans.abi
Date: 2026-05-31T04:58:57Z

## Pre-change baseline

Ran before I4 edits in this session:

```text
cargo test -p crunch-build dynamic_plan --lib
test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 396 filtered out; finished in 0.01s
```

## I4 coverage summary

- Added graph validation after scalar/limit/policy validation.
- Rejects duplicate source IDs and duplicate unit IDs.
- Rejects empty, duplicate, and unknown roots.
- Rejects undeclared `DynamicInput::Source` references.
- Rejects `DynamicInput::UnitOutput` references to unknown units or undeclared output names.
- Rejects unit-output dependency cycles with an iterative Kahn-style cycle check.

## cargo fmt --check -p crunch-build

```text

exit status: 0
```

## cargo test -p crunch-build dynamic_plan --lib

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_build-43c7f3e1499b6f6d)

running 38 tests
test dynamic_plan::tests::blake3_hex_accepts_lowercase_hex ... ok
test dynamic_plan::tests::blake3_hex_rejects_wrong_length_or_uppercase ... ok
test dynamic_plan::tests::decode_accepts_valid_plan_shape ... ok
test dynamic_plan::tests::decode_accepts_present_null_nullable_fields ... ok
test dynamic_plan::tests::decode_rejects_over_limit_json_nesting_depth ... ok
test dynamic_plan::tests::decode_rejects_non_native_derivation_sandbox ... ok
test dynamic_plan::tests::decode_rejects_missing_required_field ... ok
test dynamic_plan::tests::decode_rejects_unknown_nested_field ... ok
test dynamic_plan::tests::decode_requires_nullable_fixed_output_field ... ok
test dynamic_plan::tests::decode_rejects_unknown_top_level_field ... ok
test dynamic_plan::tests::decode_requires_nullable_goal_hint_field ... ok
test dynamic_plan::tests::canonical_bytes_are_compact_json ... ok
test dynamic_plan::tests::decode_canonical_plan_returns_canonical_plan_bytes_and_digest ... ok
test dynamic_plan::tests::output_name_rejects_invalid_grammar ... ok
test dynamic_plan::tests::decode_requires_nullable_source_digest_field ... ok
test dynamic_plan::tests::source_id_uses_unit_id_grammar ... ok
test dynamic_plan::tests::canonical_digest_ignores_formatting_and_object_key_order ... ok
test dynamic_plan::tests::output_name_accepts_expected_grammar ... ok
test dynamic_plan::tests::store_path_string_accepts_store_path_and_suffix ... ok
test dynamic_plan::tests::store_path_string_rejects_non_normal_suffix ... ok
test dynamic_plan::tests::store_path_string_rejects_wrong_prefix_and_bad_component ... ok
test dynamic_plan::tests::store_prefix_must_be_absolute_and_normalized ... ok
test dynamic_plan::tests::unit_id_accepts_expected_grammar ... ok
test dynamic_plan::tests::unit_id_rejects_invalid_grammar ... ok
test dynamic_plan::tests::canonicalization_sorts_set_like_collections ... ok
test dynamic_plan::tests::validate_rejects_policy_widening ... ok
test dynamic_plan::tests::validate_rejects_over_limit_string_field ... ok
test dynamic_plan::tests::validate_rejects_duplicate_output_names ... ok
test dynamic_plan::tests::validate_rejects_malformed_source_digest_and_absolute_host_path ... ok
test dynamic_plan::tests::validate_rejects_invalid_output_name_and_store_prefix_reference ... ok
test dynamic_plan::tests::validate_rejects_unknown_schema_version ... ok
test dynamic_plan::tests::validate_rejects_duplicate_unit_and_source_ids ... ok
test dynamic_plan::tests::validate_rejects_empty_duplicate_and_unknown_roots ... ok
test dynamic_plan::tests::validate_rejects_unit_output_dependency_cycles ... ok
test dynamic_plan::tests::validate_accepts_valid_plan_and_decode_validated_returns_canonical_digest ... ok
test dynamic_plan::tests::validate_rejects_undeclared_source_and_unit_output_refs ... ok
test dynamic_plan::tests::decode_rejects_oversized_plan_before_json_parse ... ok
test dynamic_plan::tests::validate_rejects_over_limit_units_dependencies_outputs_and_env ... ok

test result: ok. 38 passed; 0 failed; 0 ignored; 0 measured; 396 filtered out; finished in 0.01s


exit status: 0
```

## openspec validate native-dynamic-plans

```text
Change 'native-dynamic-plans' is valid

exit status: 0
```

## git diff --check

```text

exit status: 0
```

