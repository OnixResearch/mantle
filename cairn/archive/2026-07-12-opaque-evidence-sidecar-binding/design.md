## Design

Mantle release evidence receives typed metadata for external evidence sidecars. Each sidecar record carries canonical envelope identity, evidence kind, profile/schema version, upstream validation identity, source and binary artifact linkage, policy hashes, claim scope, compatibility projections, and non-claims. Mantle validates the metadata and hash chain but leaves payload parsing to Kamacite and Valence.

### Decisions

- **Binding is profile-neutral.** Function-address becomes the first profile, not a custom one-off release contract.
- **Sidecars are opaque.** Mantle validates declared sidecar metadata and digest linkage, not Rust functions, proof terms, or Preserves payload internals.
- **Release linkage is explicit.** Each sidecar binds to source artifacts and release binaries through typed hash records.
- **Policy hashes are chained.** Mantle binding commits to upstream profile/policy hashes and Mantle release policy identity.
- **Compatibility projections are bounded.** JSON sidecars can appear only as non-authoritative projections bound to canonical evidence envelope identity.
- **Pure core remains typed.** CLI hashes sidecar bytes and loads metadata; core validates in-memory release evidence.

### Validation shape

Positive fixtures cover function-address as a generic sidecar and at least one non-function-address fixture stub. Negative fixtures cover unknown evidence kind, missing canonical envelope hash, stale Valence validation hash, source/binary mismatch, unsupported claim scope, wrong role/schema, projection drift, malformed BLAKE3, and weakened non-claims.
