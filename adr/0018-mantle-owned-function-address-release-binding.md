# ADR 0018: Own the function-address release binding at the Mantle boundary

## Status

Accepted

## Context

Mantle already validates bundle-local function-address sidecars and their Valence, optional Kamacite, source-archive, and release-binary links. Cairn release readiness previously consumed an ad hoc JSON projection with field names duplicated in smoke setup. That left the cross-repository boundary versionless from Mantle's perspective and did not bind the projection back to the complete Mantle verification result.

Mantle must not parse Rust functions, reinterpret Valence evidence semantics, or parse Kamacite Preserves payloads. Cairn must remain the owner of lifecycle-policy conformance and its own non-claim boundary.

## Decision Drivers

- Give Cairn a versioned direct input without smoke-specific wrapper glue.
- Preserve Mantle's complete verification disposition and diagnostics.
- Keep lifecycle-policy scope distinct from upstream identity/linkage scope.
- Bind every projected linkage field to the embedded verification summary.
- Reject missing, malformed, stale, or weakened receipt data before handoff.
- Keep construction and validation pure, deterministic, bounded, and `no_std` compatible.

## Decision

Mantle defines `mantle.function-address-binding.v1` in `crunch-release-core`. The receipt contains Cairn's direct fields (`valid`, `disposition`, `receipt_hash`, lifecycle claim scope, sidecar/Valence/source/binary digests, and Cairn boundary), optional Kamacite linkage, exact role/schema metadata, Mantle's boundary, and the full `FunctionAddressReleaseVerification` summary.

The top-level lifecycle claim and embedded sidecar claim are intentionally different and explicitly named. The top level says only that Cairn may evaluate supplied facts against lifecycle policy. The embedded `sidecar_claim_scope` remains `function-address-identity-linkage-only`.

The receipt hash is lowercase BLAKE3 over a dedicated compact JSON material containing every receipt field except the self-referential `receipt_hash`. Validation reconstructs that material, checks exact schema and boundaries, validates bounded canonical hashes and diagnostics, enforces complete optional Kamacite metadata, compares projected links to the summary, and verifies validity/verdict/disposition coherence.

The Rust DTO is the runtime owner. A checked JSON Schema and generated Nickel contract mirror its public shape, including cross-field equality invariants. Filesystem reads and writes remain outside the core.

## Alternatives Considered

### Keep the smoke wrapper as the de facto contract

Rejected because duplicated unversioned field assembly can drift independently of Mantle verification output.

### Let Cairn parse the full Mantle release manifest

Rejected because it would couple Cairn to broader Mantle packaging internals and transfer release-manifest interpretation across repository boundaries.

### Reuse the sidecar identity/linkage claim as the top-level Cairn claim

Rejected because identity linkage and lifecycle-policy conformance are different claims with different owners.

### Include `receipt_hash` in its own hash input

Rejected because a recursive self-hash has no ordinary deterministic construction. The omitted-field hash material is explicit and independently reconstructable.

## Consequences

- Cairn can consume the Mantle receipt directly while ignoring additional operator fields it does not need.
- Any field or ownership-boundary change requires a new schema version or an explicit compatibility converter.
- A structurally complete failed verification can preserve bounded diagnostics in a deterministic `FAIL` receipt, but Cairn will not accept it as readiness evidence.
- Receipt conformance proves typed identity and linkage plus supplied lifecycle-policy facts only. It does not prove Rust semantics, compiler correctness, build correctness, verifier soundness, whole-program safety, or release eligibility.
