## Design: release-evidence provenance ID coverage

### Backward-compatible optional field

The `provenance_coverage` field is `Option<ProvenanceCoverage>` with
`#[serde(default, skip_serializing_if = "Option::is_none")]`. Existing
release-evidence manifests without the field continue to validate. The field
is populated when a release pipeline has provenance threading information
(from valence's `ProvenanceChain`) to record.

### Validation

`validate_provenance_coverage` checks:
- `binary_hash` is a valid BLAKE3 hex hash (64 lowercase hex chars)
- `coverage_boundary` matches the required non-claim boundary text
- At least one of `covered_source_ids`, `covered_function_object_ids`, or
  `covered_requirement_ids` is non-empty

### Adapter relationship

`ProvenanceCoverage` is a compact optional summary field for ID coverage that an external adapter has already produced. A Valence stack provenance adapter may generate the IDs, but Mantle treats them as opaque strings and does not parse or verify Valence, Octet, Trellis, or Cairn semantics.

For richer stack-specific evidence, Mantle should bundle an adapter-produced sidecar through the generic external evidence slot. In that flow Mantle records sidecar identity while Valence owns semantic verification.

### Non-claims

Provenance coverage records identity and linkage only. It does not prove:
- behavioral correctness
- semantic equivalence
- adapter-side semantic validity
- that the binary satisfies the requirements
- that the coverage is exhaustive

It records that a release carries adapter-supplied traceability IDs, not that those IDs are semantically sufficient for correctness or release approval.
