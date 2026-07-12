## Tasks

- [x] [serial] Define a generic opaque evidence sidecar binding contract for release provenance. r[mantle.release_provenance.opaque_evidence_sidecar_binding.contract]
  - Evidence: pure contract and canonical identity live in `crates/crunch-release-core/src/opaque_evidence.rs`; see `evidence/validation.md`.
- [x] [serial] Bind canonical envelope hash, evidence kind, profile version, Valence validation hash, source artifact hash, release binary hash, policy hashes, claim scope, and non-claims. r[mantle.release_provenance.opaque_evidence_sidecar_binding.links]
  - Evidence: manifest linkage validation and the current focused test matrix are recorded in `evidence/validation.md`.
- [x] [serial] Keep evidence payload parsing out of Mantle release validation core. r[mantle.release_provenance.opaque_evidence_sidecar_binding.opaque_core]
  - Evidence: the core accepts typed metadata and digests only; upstream payload semantics remain an explicit non-claim.
- [x] [serial] Make function-address use the generic sidecar binding path. r[mantle.release_provenance.opaque_evidence_sidecar_binding.function_address]
  - Evidence: the compatibility projection and legacy-read boundary are covered by the 161-test core run in pueue task 1604.
- [x] [serial] Add positive fixtures for function-address and a non-function-address sidecar stub. r[mantle.release_provenance.opaque_evidence_sidecar_binding.positive]
  - Evidence: pueue task 1620 passed the lifecycle-stub and multiple-profile positive fixtures.
- [x] [serial] Add negative fixtures for unknown kind, missing canonical hash, stale Valence hash, source/binary mismatch, unsupported claim scope, wrong role/schema, projection drift, malformed digest, and weakened non-claims. r[mantle.release_provenance.opaque_evidence_sidecar_binding.negative]
  - Evidence: pueue task 1620 passed the generic function-address negative fixture matrix.
- [x] [serial] Run release-core tests and stack smoke through the generic binding. r[mantle.release_provenance.opaque_evidence_sidecar_binding.validation]
  - Evidence: pueue tasks 1604, 1559, 1600, 1569, 1606, and 1610 are summarized in `evidence/validation.md`.
