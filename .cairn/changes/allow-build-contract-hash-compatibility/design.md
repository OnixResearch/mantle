## Context

Neural Stream's real shell graph rejected `mantle-build-contract` at `7126fc9`.
Mantle requires BLAKE3 `=1.8.2`. The admitted Animus session contract requires
`^1.8.5`, and the consumer lock selects 1.8.7. Standalone fixture compatibility
did not establish compatibility with that complete graph.

## Decision

Use the normal Cargo compatible range `1.8.2` for this library. Keep default
features disabled and retain `pure`. The library uses only inherent `Hasher`
and `Hash` methods. It neither enables `traits-preview` nor uses digest traits.
No request framing, schema, type, or admission function changes.

The store and other exact pins remain unchanged. The owner workspace lock must
remain on BLAKE3 1.8.2. Consumer locks own their separately admitted selections.
This change tests 1.8.2 and 1.8.7, not every future compatible version.

## Verification architecture

Two independent fixture workspaces depend on the in-repository contract and
share one consumer test source. Each workspace has a Cargo-generated lockfile
and an exact BLAKE3 version. They admit the original retained producer fixture
without regenerating it. They also reject changed identity, builder, products,
cache source, and unknown-outcome receipt data. The no-std library and both
consumers compile for WebAssembly. Nix checks run both exact fixture graphs.

These local path dependencies are same-owner test fixtures, not downstream
product dependencies. Final Neural Stream compatibility must use a published
immutable Git revision. It must retain the admitted Animus source and resolve
BLAKE3 1.8.7 without overrides. It must run linked tests and the full consumer
gate before downstream admission claims.

## Alternatives

- Downgrade Animus or force BLAKE3 1.8.2: rejected because it violates the
  already admitted consumer cohort.
- Relax the store pins: rejected because that changes digest-trait compatibility.
- Copy the identity implementation into the consumer: rejected because Mantle
  owns that framing and its conformance contract.
- Add a decoder subprocess to hide the conflict: deferred because it adds a
  new process boundary without a semantic need.

## Scope and rollback

The change is compatibility packaging and test orchestration. Its core remains
host-independent. A passing check grants no execution or release authority.
No native default CLI build is claimed at the new revision. The existing native
acceptance still names its exact previously tested source and package.
Rollback restores the library's exact pin and removes the two fixture lanes.
It must not downgrade consumer authority contracts or change retained fixtures.

## Inherited lint prerequisite

The full pinned Tiger Style check exposed three pre-existing findings in
`crunch-repair-core/src/legacy_archive.rs` from the published base. A separate
maintenance commit decomposes the existing root-bound guard and states existing
postconditions with debug assertions. It does not change rejection order,
migration decisions, signatures, content checks, or execution authority.
Focused repair-core tests precede and follow the cleanup. Additional closure
controls cover valid reachability, exact bounds, duplicates, and malformed
members. No lint is suppressed and no archived legacy change is reopened.

## Acceptance

The focused owner baseline, both matrix lanes, strict scoped Clippy, wasm,
Nickel conformance, original fixture freshness, and lifecycle gates must pass.
The owner Cargo lock, store pins, contract Rust source, and original fixture
must compare unchanged. Final evidence records the tested source and separate
published Git consumer result before task completion and archive.
