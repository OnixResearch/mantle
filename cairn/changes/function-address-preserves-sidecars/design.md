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
