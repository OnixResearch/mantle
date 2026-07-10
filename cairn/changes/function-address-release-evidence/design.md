## Context

Mantle already treats Valence stack-provenance sidecars and other external evidence as opaque bundle-local artifacts. Function-address evidence should follow the same shape: the evidence is valuable only when bound to the exact release binary and source archive, but Mantle should not parse function entries or validate source-language semantics.

The first slice should work for generic releases where the evidence is optional and for Onix stack releases where policy requires it.

## Decisions

### 1. Use the external-evidence sidecar boundary

**Choice:** Model function-address evidence as a release external-evidence sidecar with expected role, schema, claim scope, BLAKE3 byte digest, optional Kamacite receipt digest, Valence verification or graph-report digest, release binary identity, and source archive identity.

**Rationale:** Mantle already owns artifact bundling and digest verification. Reusing that mechanism avoids a second release-evidence path.

### 2. Keep function evidence opaque

**Choice:** Mantle validates sidecar presence, byte digest, metadata, role/schema, binary/source binding, and non-claim text. It does not parse individual functions or recompute function addresses.

**Rationale:** Octet owns Rust extraction, Kamacite owns portable receipt identity, and Valence owns evidence linkage semantics.

### 3. Support optional and required modes

**Choice:** Generic release profiles may record function-address evidence as absent or optional. Stack release profiles may require it and fail closed when evidence is absent, stale, or mismatched.

**Rationale:** Not every Mantle-built artifact will have Octet/Kamacite/Valence function evidence, but stack artifacts should be able to require it.

### 4. Bind both binary and source archive identities

**Choice:** Required function-address evidence must name the release binary identity and the source archive or source artifact identity when available. Mantle verifies the bundle-local identities against release metadata it already owns.

**Rationale:** Function addresses are source-derived, while release consumers care about the shipped binary. Binding both sides preserves traceability without claiming compilation correctness.

### 5. Preserve non-claim text in reports

**Choice:** Human and JSON release verification output must state that successful function-address binding is not proof of source correctness, build correctness, compiler correctness, release reproducibility, verifier soundness, or deployment safety.

**Rationale:** Bundle-local evidence can be strong provenance while still being a bounded claim.

## Risks / Trade-offs

- Required mode can block release packaging until upstream Valence/Kamacite/Octet evidence exists. Keep optional mode for generic releases.
- Binding source and binary identities may reveal source-archive workflow gaps. Fail closed in required mode rather than relaxing the claim.
- Dedicated CLI flags can wait if generic `--external-evidence` is enough for the first slice; profile constants should still be reviewed to prevent drift.
