# Nominal dynamic-plan types

Mantle keeps the `mantle-plan-v1` JSON contract structural. It admits that wire data into a typed core before graph validation.

```text
JSON bytes
  -> WireDynamicPlanV1
  -> admit_plan_v1(..., store_prefix)
  -> DynamicPlanV1
  -> graph validation and canonicalization
  -> WireDynamicPlanV1 projection
```

## Core types

The admitted model uses these private types:

- `UnitId` for unit and root identities
- `SourceId` for declared source identities
- `StorePathString` for checked logical store paths
- `OutputName` for derivation output names
- `PlanDigest` for canonical plan identity
- `NarDigest` for declared source NAR identity

Each type has a fallible constructor and an `as_str` accessor. The types do not implement `Deref` or unrestricted `From<String>` conversions.

The graph keeps these types in sets, maps, dependencies, roots, and placeholder bindings. It converts them to text only for wire output, diagnostics, and existing scheduler or store adapters.

## Wire and admission APIs

`decode_plan_v1` and `decode_wire_plan_v1` return `WireDynamicPlanV1`. These functions enforce the byte, JSON-depth, field, and required-nullable limits.

`admit_plan_v1` checks semantic scalars and returns `DynamicPlanV1`. Invalid IDs, output names, logical store paths, and NAR digests cannot enter graph validation.

`decode_validated_plan_v1` combines bounded decoding, admission, graph validation, canonicalization, and plan digest calculation.

Canonical serialization always projects the admitted model through `WireDynamicPlanV1`. This preserves the current field names, enum tags, null values, collection shapes, canonical bytes, and BLAKE3 plan identity.

## Rust consumer migration

Code that only reads JSON can continue to call `decode_plan_v1`. Its result is now the explicit wire DTO.

Code that needs graph-ready data must call `admit_plan_v1` with the active logical store prefix. Code that needs the canonical admitted result can call `decode_validated_plan_v1`.

Construct new core values with `UnitId::new`, `SourceId::new`, `StorePathString::new`, `OutputName::new`, and the role-specific digest constructors. Do not convert checked values back to strings inside core graph logic.

## Claim boundary

These types prove local category separation and scalar admission for the supplied values. They do not prove store presence, source trust, build success, sandbox enforcement, compiler correctness, or release eligibility.
