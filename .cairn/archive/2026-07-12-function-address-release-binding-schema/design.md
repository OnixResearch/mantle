## Design

The binding receipt wraps the existing Mantle release verification result in a stable schema with explicit top-level fields Cairn needs: `valid`, `verdict`, `receipt_hash`, `claim_scope`, `sidecar_digest`, `valence_receipt_digest`, `source_archive_digest`, `release_binary_digest`, optional Kamacite receipt digest, and Mantle/Cairn non-claim boundaries.

### Decisions

1. **Schema is Mantle-owned.** Mantle defines and tests the binding receipt shape; Cairn consumes it.
2. **Core output is preserved.** The receipt includes the release verification summary so operators can inspect Mantle diagnostics.
3. **Non-claims are duplicated intentionally.** Mantle's opaque boundary and Cairn lifecycle boundary are both visible.
4. **Hashes are BLAKE3.** All binding and linkage hashes use lowercase BLAKE3 hex unless an existing external artifact explicitly supplies another spelling with an adapter.

### Validation shape

Positive fixtures cover stable schema rendering and Cairn field extraction. Negative fixtures cover missing top-level fields, malformed hashes, unsupported claim scope, stale Valence digest, stale Kamacite digest, and weakened non-claim boundaries.
