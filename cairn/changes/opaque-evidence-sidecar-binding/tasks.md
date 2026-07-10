## Tasks

- [ ] [serial] Define a generic opaque evidence sidecar binding contract for release provenance. r[mantle.release_provenance.opaque_evidence_sidecar_binding.contract]
- [ ] [serial] Bind canonical envelope hash, evidence kind, profile version, Valence validation hash, source artifact hash, release binary hash, policy hashes, claim scope, and non-claims. r[mantle.release_provenance.opaque_evidence_sidecar_binding.links]
- [ ] [serial] Keep evidence payload parsing out of Mantle release validation core. r[mantle.release_provenance.opaque_evidence_sidecar_binding.opaque_core]
- [ ] [serial] Make function-address use the generic sidecar binding path. r[mantle.release_provenance.opaque_evidence_sidecar_binding.function_address]
- [ ] [serial] Add positive fixtures for function-address and a non-function-address sidecar stub. r[mantle.release_provenance.opaque_evidence_sidecar_binding.positive]
- [ ] [serial] Add negative fixtures for unknown kind, missing canonical hash, stale Valence hash, source/binary mismatch, unsupported claim scope, wrong role/schema, projection drift, malformed digest, and weakened non-claims. r[mantle.release_provenance.opaque_evidence_sidecar_binding.negative]
- [ ] [serial] Run release-core tests and stack smoke through the generic binding. r[mantle.release_provenance.opaque_evidence_sidecar_binding.validation]
