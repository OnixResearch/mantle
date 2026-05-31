# H3 required-nullable ABI field oracle

Task-ID: H3
Covers: build.engine.dynamic.plans.abi
Date: 2026-05-31T06:01:00Z

## Question

Should nullable ABI fields in `mantle-plan-v1` be optional/absent, or required-present with explicit `null` when empty?

## Inspected evidence

- `openspec/changes/native-dynamic-plans/design.md` listed `goal_hint`, `nar_blake3`, and `fixed_output` as `Option<T>` fields but also put them in the required field schema.
- Existing tests now cover `decode_requires_nullable_goal_hint_field`, `decode_requires_nullable_source_digest_field`, and `decode_requires_nullable_fixed_output_field`.
- Prior napkin correction noted that plain serde `Option<T>` accidentally accepted missing nullable ABI fields as `None`.

## Decision

`goal_hint`, `nar_blake3`, and `fixed_output` are required-present nullable ABI fields. Producers must emit the key with `null` when no value exists. Missing keys are ABI errors.

## Owner

Agent, based on implementation evidence and design-gate finding.

## Next action

Clarify design wording and validation plan, then rerun OpenSpec design/tasks gates.
