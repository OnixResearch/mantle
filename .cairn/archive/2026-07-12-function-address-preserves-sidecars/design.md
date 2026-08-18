## Design

Mantle release validation treats the Kamacite Preserves receipt as an external sidecar artifact with declared role, schema version, canonical BLAKE3 receipt hash, byte size, and optional compatibility projection hash. Mantle validates that Valence and Kamacite identities are present and bound to source/binary release artifacts, but it does not parse function records or Preserves receipt internals.

### Decisions

1. **Opaque sidecar boundary.** Mantle stores and validates sidecar metadata, not function-level Preserves values.
2. **Canonical hash is required.** Required mode needs the Kamacite canonical Preserves receipt hash and the Valence receipt hash.
3. **JSON projection is optional compatibility.** JSON sidecars can appear only when they bind to the canonical Preserves receipt identity.
4. **Release-local claims only.** Mantle verifies artifact binding, roles, schema names, claim scope, and non-claims; it does not prove Rust behavior.
5. **Pure validation core.** Release evidence validation remains over typed in-memory metadata; CLI owns file reads and digesting sidecar bytes.

### Validation shape

Positive fixtures cover required and optional release bindings with canonical Preserves receipt metadata. Negative fixtures cover missing Preserves hash, stale Valence hash, JSON projection drift, wrong role/schema, unsupported claim scope, malformed BLAKE3, source/binary mismatch, and overclaiming text.

### Implemented profile

The generic opaque-evidence DTO remains backward compatible, but the `function-address-preserves-v1` profile requires `canonical_envelope.size_bytes` and `upstream_validation.receipt_hash_blake3`. Its canonical role/schema are `kamacite-function-address-preserves-receipt` and `kamacite.function-address-preserves-receipt.v1`. The optional JSON projection uses `kamacite-function-address-json-projection` and `kamacite.function-address-receipt.v1`.

`mantle release function-address-bind --from-preserves-binding` selects exactly one typed profile from the verified release manifest. The shell performs bounded no-follow reads, verifies reopened artifact digests and size against the manifest, and parses only public Valence/projection identity-envelope fields. Canonical Preserves bytes remain opaque. The pure core checks source, binary, policy, role, schema, scope, non-claim, logical-hash, and optional projection linkage before rendering the unchanged `mantle.function-address-binding.v1` output.

A JSON projection's artifact-byte digest identifies its JSON file, while its public logical `receipt_hash` must equal the canonical Preserves BLAKE3 identity. The Valence logical `receipt_hash` remains a separate identity and its optional Kamacite link must equal that same canonical Preserves identity.
