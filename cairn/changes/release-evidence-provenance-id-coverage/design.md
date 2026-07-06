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

### Connection to valence Phase 2

Valence's `ProvenanceChain` (Phase 2 `unified-provenance-id-threading`)
threads source_id to function_object_id to requirement_ids to artifact_hash
to binary_hash. Mantle's `ProvenanceCoverage` is the final link: it records
which source_ids, function_object_ids, and requirement_ids are covered by a
specific binary_hash. Together, they answer "show me where this number comes
from" mechanically: binary → coverage → requirement → function → source.

### Non-claims

Provenance coverage records identity and linkage only. It does not prove:
- behavioral correctness
- semantic equivalence
- that the binary satisfies the requirements
- that the coverage is exhaustive

It proves the binary's behavior is *traceable* to source functions and
requirements, not that it is *correct*.
