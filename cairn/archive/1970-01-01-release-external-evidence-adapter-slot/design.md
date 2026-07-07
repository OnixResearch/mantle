## Design: release external evidence adapter slot

### Boundary

Mantle owns generic release evidence integrity. It does not own stack-specific provenance semantics. The adapter boundary is:

```text
adapter-produced sidecar bytes
  -> Mantle bundles sidecar as opaque external evidence
  -> Mantle verifies path + digest + bounded metadata
  -> adapter-specific verifier interprets sidecar schema
```

A Valence stack provenance sidecar is one consumer of this slot, not a Mantle-native concept.

### Manifest shape

Add a repeatable external evidence record with fields:

- `role`: non-empty operator-facing purpose, for example `stack-provenance-trace`.
- `schema`: non-empty sidecar schema ID, for example `valence.provenance-chain.v1`.
- `relative_path`: bundle-local path to the copied evidence file.
- `digest_blake3`: BLAKE3 digest of exact sidecar bytes.
- `claim_scope`: bounded claim for this sidecar's inclusion.
- `non_claims`: non-empty list of explicit non-claims.

Mantle validates these as opaque metadata. It does not validate `schema` against an external registry and does not parse schema-specific fields.

### CLI flow

Release creation accepts one or more external evidence inputs only when requested explicitly, for example:

```bash
mantle release create ... \
  --external-evidence stack-provenance.json \
  --external-evidence-role stack-provenance-trace \
  --external-evidence-schema valence.provenance-chain.v1 \
  --external-evidence-claim-scope identity-linkage-sidecar
```

Release verification keeps the default behavior generic:

```bash
mantle release verify target/release-evidence/<id>
```

Stack gates opt in to role presence without asking Mantle to interpret the sidecar:

```bash
mantle release verify target/release-evidence/<id> \
  --require-external-evidence-role stack-provenance-trace
```

The stack-specific semantic check remains external:

```bash
valence stack-provenance verify \
  --mantle-release target/release-evidence/<id> \
  --role stack-provenance-trace
```

### Validation

Core validation is pure over an in-memory manifest and sidecar metadata. CLI code owns file reads, copies, and digest computation.

Mantle fails closed when:

- an external-evidence path escapes the bundle;
- a sidecar is missing;
- the BLAKE3 digest mismatches;
- role, schema, or claim scope is empty;
- non-claims are empty;
- a required role is absent under an opt-in verifier flag.

Mantle accepts ordinary release evidence when no external evidence is requested.

### Relationship to existing provenance coverage

Any existing compact provenance coverage fields remain optional compatibility summaries. The generic external evidence slot is the preferred extension point for stack-specific traceability because it keeps Mantle independent from Valence/Octet/Trellis/Cairn semantics.

### Non-claims

Bundling an external sidecar proves only that exact bytes were included under a named role and digest. It does not prove the sidecar schema is valid, complete, exhaustive, sufficient for release, or semantically true.
