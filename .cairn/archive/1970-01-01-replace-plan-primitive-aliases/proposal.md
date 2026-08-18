# Proposal: Replace dynamic-plan primitive aliases

## Why

The `mantle-plan-v1` pure core defines `UnitId`, `SourceId`, `StorePathString`, and `Blake3Hex` as aliases for `String`. These aliases provide names but no compiler separation.

A source ID can enter a unit dependency. A NAR digest can enter a plan-digest field. A validated logical store path can be confused with unchecked text.

The plan parser already checks scalar grammar and limits. Mantle must preserve those checked values as distinct types after admission.

## What Changes

- Split the serialized `mantle-plan-v1` DTO from the admitted typed plan model.
- Replace primitive aliases with private newtypes and explicit accessors.
- Use marker-role digest types for plan and NAR BLAKE3 identities.
- Add typed output names and logical store paths where graph construction depends on them.
- Move scalar parsing into wire-to-core admission so invalid values cannot enter graph validation.
- Preserve canonical JSON, plan BLAKE3 identity, schema fields, and existing accepted plan bytes.
- Add compile-fail, parser, canonicalization, graph, and serialization fixtures.
- Enable the future Octet nominal-domain policy for this core after the lint is available.

## Impact

- **Core**: `crates/crunch-build/src/dynamic_plan.rs` and focused planner helpers.
- **Adapters**: Plan producers and consumers convert through explicit wire types.
- **Compatibility**: `mantle-plan-v1` JSON remains stable. Rust API changes receive compatibility constructors or a documented exact-revision migration.
- **Evidence**: Canonical-plan and graph receipts retain current identities.

## Non-goals

- Do not wrap every command argument, environment value, or display string.
- Do not change the `mantle-plan-v1` schema in place.
- Do not treat a valid store-path string as proof that the object exists.
- Do not change store, worker, scheduler, or sandbox authority.
- Do not edit vendored Nix or Snix code for this migration.
