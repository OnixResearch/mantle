# Design: Extend nominal types to trust boundaries

## Context

The dynamic-plan core proves a useful local pattern:

```text
structural wire DTO
  -> bounded decode
  -> pure semantic admission
  -> nominal core values
  -> typed validation and decisions
  -> explicit wire projection
```

Several other Mantle boundaries validate primitive values but continue to store those values as primitives. `crunch-bootstrap-core::Blake3Hex::new` is currently infallible, and derived `Deserialize` implementations can construct tuple newtypes without calling their fallible constructors. A newtype provides only a compiler category barrier when unchecked construction remains available.

This change extends nominal admission without changing the authority or claim of any affected subsystem.

## Decisions

### Decision: Separate structural input from admitted core values

**Choice:** Keep externally decoded records structural when Mantle must preserve full malformed-input diagnostics. Add a pure admission function that returns an admitted model containing checked nominal values.

Admitted types will not derive unrestricted `Deserialize`. If a boundary needs direct deserialization, its custom implementation must call the same checked constructor used by ordinary admission.

Serialization and canonical hashing will project admitted values through the accepted wire DTO.

**Rationale:** Raw input remains inspectable for bounded multi-error reporting. Invalid values cannot enter the functional core.

### Decision: Define types from semantic rules and roles

**Choice:** Add small crate-local types only when a value has a checked grammar, bound, unit, authority class, or substitution risk. Each type will have a fallible constructor, explicit accessor, and no unrestricted `Deref` or `From<String>` implementation.

Do not add one repository-wide `Id<T>` or `StringValue<T>` abstraction. Share a type only when its grammar, compatibility contract, and error semantics are identical.

**Rationale:** Concrete names preserve domain meaning and useful diagnostics without coupling unrelated crates.

### Decision: Use checked digest storage with selected role types

**Choice:** Use one checked lowercase BLAKE3 representation inside each owning core. Add role markers only for values that can reach the wrong same-format API, such as plan, artifact, provider, audit, and release identities.

Algorithm-tagged interoperability values remain an enum or equivalent checked sum type. Mantle will not represent algorithm, digest text, and interoperability reason as independent values after admission.

**Rationale:** Format validation belongs in one place. Selected role separation prevents evidence substitution without creating a marker type for every display field.

### Decision: Admit and resolve bootstrap identities

**Choice:** StageX will use admitted stage, artifact, executable-authorization, safe-absolute-path, limit, and digest types. Bootstrap lineage declarations will use role-specific source, generated-artifact, tool, patch, and assumption identifiers.

Lineage graph admission will resolve raw node text into an enum such as seed, source, generated artifact, tool, or patch. Graph logic will not use an unresolved string as a node reference.

**Rationale:** The graph must retain both node existence and node role after validation.

### Decision: Keep remote protocol identities compatible

**Choice:** Add bounded `RemoteRequestId`, `RemoteProtocolSessionId`, and `RemoteEndpointId` types for admitted protocol and coordinator logic. Do not require BLAKE3 spelling unless the accepted wire contract already requires it.

Ticket identifiers, bearer tokens, verifier values, validity windows, use limits, build-time limits, and upload limits remain owned by `harden-remote-credential-boundary`. This change consumes those types after that dependency lands.

**Rationale:** Existing fixtures and compatibility inputs use non-digest identifiers. Nominal typing must not introduce an undocumented wire restriction.

### Decision: Preserve aggregate invariants in aggregate types

**Choice:** Use unit-bearing scalar types for milliseconds, seconds, byte limits, counts, offsets, and generations when accidental substitution is plausible. Use aggregate types when two same-unit values have a relationship, such as a validity window.

Standard-library shells may convert admitted timeout values to `Duration`. No-std cores retain explicitly named fixed-width representations.

**Rationale:** A `UnixSeconds` newtype alone cannot prove that start and expiry values are ordered.

### Decision: Use role-specific paths and request records

**Choice:** Add distinct types for safe absolute executable paths, specification paths, repository-relative paths, bundle-relative paths, logical store prefixes, provisional store paths, and final store paths where existing rules differ.

Use request records for operations with several semantic inputs. A Git fetch request will group repository URL, revision, archive or checkout source, and output directory instead of relying on several `&str` arguments.

**Rationale:** Path spelling does not establish the same authority at each boundary. Request records also keep imperative shells thin as operations gain policy fields.

### Decision: Migrate in bounded ownership slices

**Choice:** Apply the pattern in this order:

1. StageX digest, identifier, path, and limit admission.
2. Bootstrap lineage declarations and resolved graph references.
3. Remote protocol request, session, and endpoint identities.
4. Content-bound release and frontend artifact identities, paths, and digests.
5. Fetch, content-addressed planning, rewrite, and pipeline helper boundaries.

Each slice must keep its core pure and retain a narrow compatibility adapter in the shell or wire module.

**Rationale:** Small ownership slices permit focused review and prevent a repository-wide mechanical wrapper migration.

### Decision: Prove category safety and wire stability separately

**Choice:** Add positive constructors and admission tests, negative malformed and boundary tests, compile-fail role-substitution tests, and golden serialized-byte and digest comparisons.

Tests must include direct malformed Serde input where a type implements deserialization. A passing constructor test alone is insufficient.

**Rationale:** Compiler separation and wire compatibility are independent claims.

## Functional core and imperative shell

Pure cores own constructors, admission, reference resolution, limit relationships, digest-role conversion, and deterministic wire projection. They do not read files, inspect environment state, access clocks, perform network I/O, spawn processes, or print diagnostics.

Shells own JSON or protocol byte reads, filesystem paths, URL transport, remote sessions, explicit time observations, output sinks, and compatibility API adaptation.

## Risks and trade-offs

- Wire-to-core conversion adds code and allocation. It creates one reviewable trust boundary.
- Direct Rust callers can break. Compatibility adapters must remain until an exact migration is documented.
- New scalar bounds can change compatibility. Constructors must reuse accepted rules unless a separate spec approves new limits.
- Too many digest roles can obscure logic. Role markers are limited to demonstrated substitution risks.
- Derived Serde can silently bypass constructors. Admitted types require custom Serde or explicit wire admission.
- A typed path or digest proves local shape and role only. It does not prove external state or authority.

## Claim boundary

This change proves local scalar admission, category separation, selected relationship invariants, and preserved accepted wire identity. It does not prove artifact correctness, source trust, store presence, safe filesystem I/O, sandbox enforcement, remote peer trust, compiler correctness, or release eligibility.
