# Cold 1736 compiler-action admission: templated compiles

## Observed failure

Genuine cold task `1736` ran the reviewed launch authority
`59ac0760…` on Leviathan with the exact frozen product
`70db08685c0c5fa5c326bfe32db78576d7e2e182`. Every driver preflight passed:
source tree, runner, profile and preparation bytes; GNU tar, bubblewrap and
static-busybox pins; a real `--unshare-all --unshare-user` smoke; all three
success aliases absent; and a `ready` complete source-bundle verification.

It failed closed after 4h10m of pre-execution admission:

```text
source-built Mantle actions: unit 101:registry+https://github.com/rust-lang/crates.io-index#proc-macro-hack@0.5.20+deprecated:proc-macro-hack:proc-macro:build has no actual source-produced compiler action
```

Plan `268cd29da41f47b41354fbf9998d6c4214d8842e84b3700e9aff777076daed3e` and
the staging tree are retained. `native-store` is empty, no execution root
exists, and the success aliases stayed absent: no provider or Mantle stage ran.
`plan_actions` admits both Mantle programs and both Cargo-free inventories
before either stage executes.

## Root cause

Authoring moves every build-metadata compile's words out of the compiler action:

- `append_metadata_invocation` takes the compiler `Exec`'s arguments and
  environment, writes them into a `WriteText` template, appends a helper action
  (`metadata-helper --compiler-invocation <template> <actual-metadata:compiler.env>
  <invocation> <environment-data>`), and leaves the compiler action with empty
  arguments/environment plus `invocation_input = <step>-invocation`.
- The helper copies the template's words one-for-one: non-empty literal parts
  become same-byte data slices of `environment-data`, and typed parts are
  unchanged. Only the environment is extended from the actual metadata frame.
  Build-script flags reach the compiler through the typed
  `@metadata/compiler.rsp` input.

Admission read only the compiler action's literal arguments. Each frozen Mantle
stage has 930 direct and 119 templated metadata-bearing compiler actions, so all
119 templated units in each stage were inadmissible. `proc-macro-hack` was
simply first in topology order; the observed unit's compile action is
`unit-00002-compile`, whose words live in
`unit-00002-compile-invocation-template` with
`metadata=d4deb81969bf70e4`.

Both inert inspectors and both independent reviews accepted these units through
a static-template projection that the frozen consumer never implemented. A
checker that models a consumer's rule instead of exercising the consumer's code
can accept actions the consumer rejects.

## Repair

`source_built_mantle_programs_metadata` now owns the compiler-invocation
vocabulary and one `CompilerInvocationTemplate` type that authoring serializes
and admission strictly deserializes (unknown fields rejected).
`source_built_fixed_point_mantle_actions` resolves the words each compiler
action actually receives:

- a direct action contributes its literal arguments unchanged;
- a templated action contributes its template's arguments only when its own
  arguments and environment are empty, it binds `<step>-invocation#invocation`
  and `<step>-invocation-data#environment-data`, its helper is the
  stage-produced executable run with exactly the authored
  `--compiler-invocation` operands over `<step>-invocation-template#template`,
  that template action is a stage-local `WriteText` with no inputs, and the
  template's `data_name`, inputs, outputs and runtime mounts equal exactly this
  compile action's own typed scope.

Unit selection and `validate_primary_source` then use those resolved words.
Direct compilation, the runtime executor, the helper source, typed-input
admission and source hashing are unchanged.

ADR 0161 records the boundary.

## Verification

### Focused tests

```text
running 6 tests
test source_built_fixed_point_mantle_actions::tests::templated_compile_admits_the_authored_template_words ... ok
test source_built_fixed_point_mantle_actions::tests::templated_compile_rejects_a_substituted_helper_template ... ok
test source_built_fixed_point_mantle_actions::tests::templated_compile_rejects_its_own_literal_words ... ok
test source_built_fixed_point_mantle_actions::tests::templated_compile_rejects_template_scope_drift ... ok
test source_built_fixed_point_mantle_actions::tests::templated_primary_source_requires_its_exact_typed_member ... ok
test source_built_fixed_point_mantle_actions::tests::templated_probe_without_a_metadata_word_is_admitted_but_not_a_compiler_unit_action ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2983 filtered out; finished in 0.00s
```

```text
running 1 test
test source_built_mantle_programs::metadata::tests::compiled_helper_matches_the_shared_compiler_invocation_contract ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2988 filtered out; finished in 0.00s
```

Commands: `cargo test -p mantle --bin mantle templated_ -- --test-threads=1` and
`cargo test -p mantle --bin mantle compiled_helper_matches_the_shared_compiler_invocation_contract -- --test-threads=1`.

### Frozen-cohort rehearsal

Run over the actual frozen stage recipes and unit plans copied from the 70db0868
cohort, byte-verified as `296bc261…` (stage1 recipe), `a78ee751…` (stage2
recipe), `705d6f40…` (stage1 unit plan) and `9d20212a…` (stage2 unit plan):

```text
test source_built_fixed_point_mantle_actions::tests::frozen_stage_recipes_admit_one_compiler_action_per_unit ... mantle-stage1: rustc_actions=1121 units=1047 templated_units=119
mantle-stage2: rustc_actions=1121 units=1047 templated_units=119
ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2988 filtered out; finished in 43.69s
```

Command: `MANTLE_COLD_RECIPE_DIR=<frozen dir> cargo test -p mantle --bin mantle frozen_stage_recipes_admit_one_compiler_action_per_unit -- --ignored --nocapture --test-threads=1`.

Every unit resolves exactly one compiler action selected by that unit's own
frozen `rustc_metadata_hash`, no action serves two units, all 1,121 rustc
actions per stage resolve their authored words, and both direct (928) and
templated (119) unit paths are exercised.

### Strict template invariants

`templated-compiler-action-invariants.jq` checks all twelve strict rules against
the same frozen bytes. Output `templated-compiler-action-invariants.json`
(BLAKE3 `44647575a487c8e49f643a5446a04602088f216eee5df74a1e33c21b3a73fc94`):

```json
{"stage":"mantle-stage1","templated":188,"failures":[]}
{"stage":"mantle-stage2","templated":188,"failures":[]}
```

### Formatting

`cargo fmt --check -p mantle` passed after formatting the two edited leaf
modules.

## Scope

This proves the repaired admission resolves and selects compiler actions
correctly on the frozen cohort's own bytes and that authoring and admission
share one template contract. It does not prove any protected execution,
provider or stage success, stage equality, root action-trust reconciliation, a
verified v2 receipt, native-row qualification or release. A changed source tree
still requires a new frozen source, separate absent-target qualifications,
complete source-only regeneration, fresh full current-data independent review
and a fresh protected attempt.
