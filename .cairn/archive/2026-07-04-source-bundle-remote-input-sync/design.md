## Context

Source bundles and imported source state already model source/input availability independently from build outputs. Remote build input sync needs the same identity model so missing source inputs can be uploaded from verified state instead of requiring the client's physical store path or a live network fetch.

## Decisions

### 1. Source refs are first-class remote input refs

**Choice:** Remote input manifests include source-bundle records, imported-source-state identities, PathInfo refs, and CAS object refs as separate classes with deterministic BLAKE3 identities and named limits.

**Rationale:** Builders should request the exact missing class they lack, and clients should upload only verified matching material.

### 2. Import state can satisfy remote upload

**Choice:** A verified imported source record may materialize upload artifacts for a remote builder when its source identity, digest, store prefix, and readiness class match the requested manifest.

**Rationale:** This closes the remote-builder handoff gap for offline/pre-bundled sources.

### 3. Privacy policy runs before bytes move

**Choice:** Upload planning produces a redacted summary of source, store, proof, and secret-descriptor classes with bounded object and byte counts before route selection or transfer.

**Rationale:** Operators must reject disallowed upload classes before a remote session observes content.

## Risks / Trade-offs

- Some source kinds may remain unsupported until their adapter can produce deterministic source records.
- Materializing source state for upload can be expensive; plans need explicit size limits and fail-closed overflow behavior.
