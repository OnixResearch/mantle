# Design: Replace dynamic-plan primitive aliases

## Context

`dynamic_plan.rs` is a pure Rust core for decoding, checking, canonicalizing, hashing, and graph validation. It currently deserializes directly into models whose semantic aliases all resolve to `String`.

The parser checks bounds and scalar grammar later. A checked value loses that admission fact because its Rust type remains unchanged.

## Decisions

### Decision: Separate wire and admitted plans

**Choice:** Add `WireDynamicPlanV1` and related wire DTOs that preserve the current Serde shape. Convert them through one pure admission function into typed `DynamicPlanV1` values.

`decode_plan_v1` remains the bounded decode surface. A compatibility function can expose the raw DTO if an existing internal caller requires it.

**Rationale:** External JSON remains structural. The graph core receives only admitted values.

### Decision: Replace direct primitive aliases

**Choice:** Add private newtypes for `UnitId`, `SourceId`, `StorePathString`, and `OutputName`. Keep `StorePathString` as the reviewed compatibility name for a logical store path. Reuse the current named limits and scalar rules in fallible constructors.

Add explicit `as_str` accessors. Do not implement `Deref` or unrestricted `From<String>` for refined values.

**Rationale:** The compiler prevents category exchange, and constructors preserve existing invariants.

### Decision: Type digest roles

**Choice:** Add one private BLAKE3 value and marker aliases for `PlanDigest` and `NarDigest`.

Keep fixed-output hashes in their algorithm-tagged existing model because SHA algorithms remain interoperability requirements.

**Rationale:** Both Mantle-owned values use BLAKE3 text but represent different facts.

### Decision: Keep canonical JSON stable

**Choice:** Serialize the admitted model through an explicit wire projection. Compare its bytes and digest with the current baseline.

Transparent scalar wrappers can be used internally, but canonical JSON authority remains the versioned wire projection.

**Rationale:** Rust type safety must not change plan identity.

### Decision: Make graph APIs role-specific

**Choice:** Index units by `UnitId`, sources by `SourceId`, outputs by `(UnitId, OutputName)`, and source NAR evidence by `NarDigest`.

Avoid converting back to strings inside graph validation. String conversion is limited to diagnostics and wire output.

**Rationale:** Early string conversion discards the new compiler guarantee.

### Decision: Prove compatibility and category separation

**Choice:** Add compile-fail fixtures for unit/source, plan/NAR digest, store-path/source, and output/unit substitutions.

Add golden fixtures for current JSON, canonical order, plan digest, placeholder parsing, and errors.

**Rationale:** Both the new guarantee and the unchanged contract need direct evidence.

## Functional core and shell boundary

Wire parsing, admission, canonicalization, graph checking, and digest calculation remain pure.

File reads, builder execution, store lookup, scheduling, and report publication remain outside the dynamic-plan core.

## Test design

Positive tests cover valid construction, wire admission, placeholders, graph edges, canonical ordering, serialization, and existing plan fixtures.

Negative tests cover empty and oversized IDs, control characters, invalid store paths, malformed BLAKE3 values, wrong digest roles, missing references, duplicate IDs, placeholder role errors, and compile-time substitutions.

## Risks and trade-offs

- The wire/core split adds conversion code. It also gives one clear trust boundary.
- Rust callers can break. Compatibility adapters and exact-revision migration notes limit the effect.
- Canonical bytes can drift through innocent Serde changes. Golden tests block that drift.
- A typed logical path does not prove store presence. Existing store admission remains separate.

## Claim boundary

The change proves local plan-domain separation, checked scalar construction, and preserved wire identity. It does not prove build success, store presence, sandboxing, or release eligibility.
