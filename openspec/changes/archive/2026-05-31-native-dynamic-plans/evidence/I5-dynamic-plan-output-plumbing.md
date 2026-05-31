# I5 dynamic-plan output plumbing

Task-ID: I5
Covers: build.engine.dynamic.plans.declared.outputs, defaults.dynamic.derivations
Date: 2026-05-31T05:22:04Z

## Coverage summary

- Added `dynamic_plan_outputs | Array String | default = []` to `lib/derivation.ncl`.
- Added `dynamic_plan_outputs` passthrough to `builders/mk_derivation.ncl`.
- Added `CrunchDerivation.dynamic_plan_outputs: Vec<String>` and deserialization defaults.
- Validates each declared dynamic-plan output is present in `outputs` and rejects duplicate declarations.
- Preserves `dynamic_plan_outputs` through `ConversionCache::iter_entries()` / `drain_pending()` and `DerivationRegistry` / worker eval messages.
- Keeps dynamic-plan declarations out of Nix derivation hashing inputs; conflicting metadata for a cached derivation identity is rejected.

## cargo fmt --check and diff check

Command:

```sh
cargo fmt --check -p crunch-glue -p crunch-build -p crunch-eval -p mantle
git diff --check
```

Output:

```text

exit status: 0
```

## cargo test -p crunch-glue --lib

Command:

```sh
cargo test -p crunch-glue --lib
```

Output excerpt:

```text
running 77 tests
...
test result: ok. 77 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

exit status: 0
```

## cargo test -p crunch-eval dynamic_plan_output --lib

Command:

```sh
cargo test -p crunch-eval dynamic_plan_output --lib
```

Output excerpt:

```text
running 3 tests
test tests::derivation_contract_accepts_declared_dynamic_plan_outputs ... ok
test tests::derivation_contract_defaults_dynamic_plan_outputs ... ok
test tests::derivation_deserialization_rejects_undeclared_dynamic_plan_output ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 73 filtered out; finished in 0.04s

exit status: 0
```

## cargo test -p crunch-build registry --lib

Command:

```sh
cargo test -p crunch-build registry --lib
```

Output excerpt:

```text
running 28 tests
...
test registry::tests::insert_with_dynamic_plan_outputs_preserves_metadata ... ok
test registry::tests::populate_registry_from_iterator ... ok
test worker::tests::streaming_entries_populate_registry_on_arrival ... ok
...
test result: ok. 28 passed; 0 failed; 0 ignored; 0 measured; 407 filtered out; finished in 0.03s

exit status: 0
```

## cargo test -p crunch-build dynamic_plan --lib

Command:

```sh
cargo test -p crunch-build dynamic_plan --lib
```

Output excerpt:

```text
running 39 tests
...
test dynamic_plan::tests::validate_rejects_undeclared_source_and_unit_output_refs ... ok
test dynamic_plan::tests::decode_rejects_oversized_plan_before_json_parse ... ok
test dynamic_plan::tests::validate_rejects_over_limit_units_dependencies_outputs_and_env ... ok

test result: ok. 39 passed; 0 failed; 0 ignored; 0 measured; 396 filtered out; finished in 0.01s

exit status: 0
```

## cargo check -p mantle --bin mantle

Command:

```sh
cargo check -p mantle --bin mantle
```

Output excerpt:

```text
warning: `mantle` (bin "mantle") generated 14 warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.41s

exit status: 0
```

## openspec validate native-dynamic-plans

Command:

```sh
openspec validate native-dynamic-plans
```

Output:

```text
Change 'native-dynamic-plans' is valid

exit status: 0
```
