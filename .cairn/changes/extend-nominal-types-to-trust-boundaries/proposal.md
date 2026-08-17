# Change: Extend nominal types to trust boundaries

## Why

Mantle already separates structural `mantle-plan-v1` wire data from a nominal admitted core. Other trust-sensitive boundaries still keep identifiers, paths, digests, limits, and timestamps as raw `String`, `u32`, or `u64` values after validation.

This loses category information after a check succeeds. It also permits direct Serde construction to bypass fallible constructors, makes unrelated values interchangeable, and leaves several functions under `tigerstyle::ambiguous_params` exceptions.

The highest-risk gaps are StageX and bootstrap lineage, remote protocol identities, content-bound release evidence, frontend artifact admission, and content-addressed fetch or rewrite helpers.

## What Changes

- Extend the reviewed structural-wire to admitted-core pattern beyond dynamic plans.
- Add checked crate-local types for semantic identifiers, safe path classes, unit-bearing limits, and lowercase BLAKE3 values.
- Add selected role-specific digest types where same-format digest substitution can change evidence meaning.
- Resolve bootstrap lineage references into typed node roles before admitted graph logic uses them.
- Keep remote request, session, endpoint, and transfer identities distinct from each other and from credential values.
- Keep ticket bearer material, verifiers, and credential policy values under the existing `harden-remote-credential-boundary` change.
- Replace selected ambiguous helper signatures with typed request records or role-specific path and name types.
- Preserve current JSON shapes, canonical bytes, BLAKE3 identities, diagnostic coverage, and compatibility adapters.
- Add positive, negative, compile-fail, malformed-Serde, and golden compatibility tests.

## Non-Goals

- Wrapping every string, number, command argument, environment value, display label, or diagnostic path.
- Changing an accepted wire schema without a separate versioned compatibility change.
- Treating a checked path as proof of filesystem presence, authority, or safe I/O.
- Treating a checked digest as proof of artifact correctness, trust, or release eligibility.
- Replacing algorithm-tagged interoperability digests with BLAKE3-only types.
- Creating one global generic identity type that hides domain-specific validation or diagnostics.
- Duplicating ticket entropy, verifier, migration, or secret-delivery work from `harden-remote-credential-boundary`.

## Dependencies

- The accepted `build_correctness.dynamic_plan_nominal.*` requirements and `docs/nominal-dynamic-plan-types.md` define the reference boundary.
- `harden-remote-credential-boundary` owns ticket bearer, verifier, and credential-policy nominal types.
- The generated Cairn policy must be refreshed for the current `nominal_identity_policy` schema before lifecycle gates can pass.

## Impact

- **Affected specs:** `build-correctness`
- **Affected core code:** `crunch-bootstrap-core`, `crunch-build`, `crunch-release-core`, and selected project-core value boundaries
- **Affected shell code:** remote protocol admission, frontend artifact admission, fetch adapters, and compatibility projections
- **Compatibility:** accepted JSON and canonical identity remain unchanged unless a separate versioned contract approves a change
- **Testing:** focused core tests, malformed wire tests, compile-fail role tests, canonical golden tests, first-party quality rails, and Cairn gates
