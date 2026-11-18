# 0161. Admit build-metadata compiler actions through their authored template

Status: Accepted

## Context

Leviathan cold task 1736 failed closed in pre-execution admission on
`proc-macro-hack@0.5.20+deprecated`'s proc-macro unit: it had "no actual
source-produced compiler action". The proof plan
`268cd29da41f47b41354fbf9998d6c4214d8842e84b3700e9aff777076daed3e` and its
staging tree are retained; `native-store` is empty and no success alias exists.
Every runtime preflight before admission had passed.

Authoring (`append_metadata_invocation`) moves a build-metadata compile's words
out of its compiler action and into a `WriteText` template. The compiler action
keeps empty arguments and an `invocation_input` naming a metadata-helper action
that reads the template plus the actual `compiler.env` frame and writes the
invocation the executor decodes. The helper copies the template's words
one-for-one: non-empty literal parts become same-byte data slices and typed
parts are unchanged. Build-script flags reach the compiler through the typed
`@metadata/compiler.rsp` input, not through rewritten words.

Admission read only the compiler action's literal arguments. Each Mantle stage
has 930 direct and 119 templated metadata-bearing compiler actions, so all 119
templated units per stage were inadmissible; `proc-macro-hack` was simply first
in topology order. Both inert inspectors and both independent reviews accepted
these units through a static-template projection that the frozen consumer never
implemented. A checker that models a consumer's rule instead of exercising the
consumer's code can accept actions the consumer rejects.

## Decision Drivers

- The runtime executor consumes only the helper-produced invocation.
- Admission must prove which source-authored words reach the compiler.
- Authoring and admission must not hold two descriptions of one format.
- Preserve exact direct-compile admission and exact primary-source comparison.
- Fail closed on any substituted helper, template, or typed scope.
- Do not execute helpers or products to decide admission.

## Decision

`source_built_mantle_programs_metadata` owns one
`CompilerInvocationTemplate` type plus the compiler-invocation vocabulary:
mode, template/output/data identity names, the compile-action identity helper
(`<step>-invocation`, `<step>-invocation-template`, `<step>-invocation-data`)
and the helper's exact operand list. Authoring serializes that type; admission
deserializes the same type with unknown fields rejected.

`compiler_words` resolves the words a compiler action actually receives. A
direct action contributes its literal arguments unchanged. A templated action
contributes its template's `invocation.arguments`, and only after:

- its own arguments and environment are empty, so no word is silently ignored;
- it reads the `<step>-invocation` output and the `<step>-invocation-data`
  output of the action named by its `invocation_input`;
- that helper is the stage-produced executable run with exactly the authored
  `--compiler-invocation` operands and reads the `<step>-invocation-template`
  text;
- the template action is a stage-local `WriteText` with no inputs whose output
  is the authored template name;
- the template's `data_name`, inputs, outputs and runtime mounts equal exactly
  that compile action's own typed inputs (excluding the invocation and data
  bindings), outputs and runtime mounts.

Unit selection then matches each Cargo-free unit against these resolved words,
and `validate_primary_source` compares the unit's planned primary source with a
typed word from the same resolved set. Direct compiles are unchanged. The
runtime executor, helper source, typed-input admission and source hashing are
unchanged.

## Consequences

- Authoring and admission share one contract, so a format change cannot move
  silently on one side. A helper or template that is not the authored one is
  rejected instead of projected.
- Admission proves the authored provenance and typed scope of the words; it
  does not execute or verify the helper's own I/O behaviour, which remains a
  measured protected-action obligation.
- The current consumer is the source-built cold fixed-point proof. Changed
  consumer bytes require a new published frozen source, separate absent-target
  qualifications, complete source-only regeneration, fresh full current-data
  review and a fresh protected attempt.
- The retained failed staging directory and both frozen stage recipes and unit
  plans support an opt-in rehearsal (`MANTLE_COLD_RECIPE_DIR`) that runs this
  real admission over the frozen bytes without preparing prior recipes or
  executing anything. It selects each unit by that unit's own frozen
  disambiguator.
  The rehearsal reports observed counts and asserts only structural facts
  (every unit resolves one action; no action serves two units; templated and
  direct paths are both exercised), never a previous run's totals.
- Inert inspection is not provider execution, a fixed point, a verified v2
  receipt, native-row qualification or release authority.
