# V1 dynamic-plan core verification

Task-ID: V1
Covers: build.engine.dynamic.plans.abi
Date: 2026-05-31T05:58:58Z

Command:

```sh
cargo test -p crunch-build dynamic_plan --lib
```

Output excerpt:

```text
running 46 tests
test dynamic_plan::tests::decode_accepts_valid_plan_shape ... ok
test dynamic_plan::tests::canonical_digest_ignores_formatting_and_object_key_order ... ok
test dynamic_plan::tests::decode_requires_nullable_fixed_output_field ... ok
test dynamic_plan::tests::decode_requires_nullable_goal_hint_field ... ok
test dynamic_plan::tests::decode_requires_nullable_source_digest_field ... ok
test dynamic_plan::tests::validate_rejects_policy_widening ... ok
test dynamic_plan::tests::validate_rejects_unit_output_dependency_cycles ... ok
test dynamic_plan::tests::validate_rejects_undeclared_source_and_unit_output_refs ... ok
test dynamic_plan::tests::validate_accepts_valid_plan_and_decode_validated_returns_canonical_digest ... ok
test dynamic_plan::tests::decode_rejects_oversized_plan_before_json_parse ... ok
test dynamic_plan::tests::validate_rejects_over_limit_units_dependencies_outputs_and_env ... ok
...
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 397 filtered out; finished in 0.02s

exit status: 0
```
