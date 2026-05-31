# I3 scalar, limit, and policy validation

Task-ID: I3
Covers: build.engine.dynamic.plans.abi
Date: 2026-05-31T04:52:23Z

## Pre-change baseline

Ran before I3 edits in this session:

```text
cargo test -p crunch-build dynamic_plan --lib
test result: ok. 24 passed; 0 failed; 0 ignored; 0 measured; 396 filtered out; finished in 0.00s
```

## Review gap closure

- Added explicit `DynamicDerivation.sandbox` validation call.
- Documented sandbox enforcement boundary: `SandboxMode` is a closed serde enum, so non-`native` sandbox JSON is rejected during decode before the validator can receive a widened enum value; validator logic accepts only the decoded `Native` variant.
- Added a negative decode test for non-`native` sandbox JSON.
- Added duplicate output-name rejection for unit outputs, requested outputs, and dynamic plan outputs, with focused negative tests.

## cargo fmt --check -p crunch-build

```text

exit status: 0
```

## cargo test -p crunch-build dynamic_plan --lib

```text
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_build-43c7f3e1499b6f6d)

running 34 tests
test dynamic_plan::tests::blake3_hex_accepts_lowercase_hex ... ok
test dynamic_plan::tests::blake3_hex_rejects_wrong_length_or_uppercase ... ok
test dynamic_plan::tests::decode_accepts_valid_plan_shape ... ok
test dynamic_plan::tests::decode_accepts_present_null_nullable_fields ... ok
test dynamic_plan::tests::decode_rejects_missing_required_field ... ok
test dynamic_plan::tests::decode_rejects_over_limit_json_nesting_depth ... ok
test dynamic_plan::tests::canonical_bytes_are_compact_json ... ok
test dynamic_plan::tests::decode_canonical_plan_returns_canonical_plan_bytes_and_digest ... ok
test dynamic_plan::tests::decode_rejects_unknown_nested_field ... ok
test dynamic_plan::tests::output_name_accepts_expected_grammar ... ok
test dynamic_plan::tests::output_name_rejects_invalid_grammar ... ok
test dynamic_plan::tests::source_id_uses_unit_id_grammar ... ok
test dynamic_plan::tests::decode_requires_nullable_fixed_output_field ... ok
test dynamic_plan::tests::canonical_digest_ignores_formatting_and_object_key_order ... ok
test dynamic_plan::tests::decode_requires_nullable_goal_hint_field ... ok
test dynamic_plan::tests::decode_rejects_non_native_derivation_sandbox ... ok
test dynamic_plan::tests::decode_requires_nullable_source_digest_field ... ok
test dynamic_plan::tests::decode_rejects_unknown_top_level_field ... ok
test dynamic_plan::tests::store_path_string_accepts_store_path_and_suffix ... ok
test dynamic_plan::tests::store_path_string_rejects_non_normal_suffix ... ok
test dynamic_plan::tests::canonicalization_sorts_set_like_collections ... ok
test dynamic_plan::tests::store_path_string_rejects_wrong_prefix_and_bad_component ... ok
test dynamic_plan::tests::store_prefix_must_be_absolute_and_normalized ... ok
test dynamic_plan::tests::unit_id_accepts_expected_grammar ... ok
test dynamic_plan::tests::unit_id_rejects_invalid_grammar ... ok
test dynamic_plan::tests::validate_rejects_over_limit_string_field ... ok
test dynamic_plan::tests::validate_rejects_invalid_output_name_and_store_prefix_reference ... ok
test dynamic_plan::tests::validate_rejects_malformed_source_digest_and_absolute_host_path ... ok
test dynamic_plan::tests::validate_rejects_policy_widening ... ok
test dynamic_plan::tests::validate_rejects_unknown_schema_version ... ok
test dynamic_plan::tests::validate_rejects_duplicate_output_names ... ok
test dynamic_plan::tests::validate_accepts_valid_plan_and_decode_validated_returns_canonical_digest ... ok
test dynamic_plan::tests::decode_rejects_oversized_plan_before_json_parse ... ok
test dynamic_plan::tests::validate_rejects_over_limit_units_dependencies_outputs_and_env ... ok

test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 396 filtered out; finished in 0.01s


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

