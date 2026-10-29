# Design

## Context

See ADR 0136 and `docs/wasm-component-effects.md` for the ownership split, schema,
fixture byte contract, and exact production commands.

## Decisions

`ComponentPipelineRequest.runtime_validation` defaults to standalone smoke. A
consumer-host-required value must exactly match the manifest WIT package/world
and planned remaining imports, with no standalone invoke/stdout settings. The
artifact still passes every static admission stage. A `not-run` graph node binds
the final artifact and runtime profile without a tool or execution claims.

The bundle exports the same typed policy and includes it in consumer-mode
identity. Its existing exact WIT member binds the complete versioned package,
not merely a namespace/name string. Structural verification rejects missing or
wrong-artifact requirement references and producer smoke references in consumer
mode; complete verification remeasures every declared object. Runtime consumers
must independently operate their actual host and bind receipts to the admitted
bundle, artifact, runtime profile, WIT, and checker cohort.

The effect source is dependency-free `no_std` Rust. The pinned producer generates
bindings from reviewed exact WIT bytes without editing generated text. Explicit
input modes select forwarding, pure response, trap, malformed output, and
fuel/deadline computation; no clock/environment/random source selects behavior.
A bounded reclaiming allocator supports repeated invocation without monotonic
arena exhaustion. No unused WIT package is added to make the package closure
nonempty. Exact empty-lock validation remains mandatory.

Octet's new structural profile and admitted seed are independent checker-owned
sources. Mantle pins the immutable checker revision, full catalog digest, default
profile identity, wasm-tools cohort, and exact selectable profile names. Per-stage
Octet validation binds the requested profile identity and never falls back to a
baseline profile. The integrated checker revision is published immutable
`abb6082062251944a40093a274df3d35331fa168`; its measured generated identities,
not preparation placeholders, supply the coherent product pin and constants.

## Rejected Alternatives

Standalone mock imports, a successful skipped Wasmtime process, a synthetic WIT
dependency, generated-binding text rewriting, and broadening either baseline
profile would erase an ownership or identity boundary. A sibling checker source
override is not an immutable product dependency. None is an admitted fallback.

## Verification

Run the focused core/adapter/verifier and actual CLI regressions, including
standalone success and missing/duplicate/unknown/misbound selection denials.
Generate both genuine source closures, run the actual bounded CLI, verify its
report, and independently remeasure every bundle member through Structural+Bytes
and source-copy verification. Preserve failed attempts and exact final commands.
Real actor/extension execution belongs to Molten and is cited separately; a
producer success never asserts that its host requirement was fulfilled.

## Risks, Ownership, and Rollback

Mantle owns typed pure policy and canonical bundle identity in the core; its std
shell owns filesystem/tool effects and exact ordered stage evidence. Closed
profile names and identities, exact WIT bytes, and canonical member ordering
must remain coherent across source, lock, toolchain generation, and publication.
Unknown profiles or contradictory runtime policy fail before tool execution;
static stage failure prevents an admitted bundle. Roll back the complete source,
lock, constants, and cohort together, never relabeling existing bundles. Octet
owns checker policy and Molten owns runtime authority and downstream receipts.
