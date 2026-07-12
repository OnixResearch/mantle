## Tasks

- [x] [serial] Register Trellis proof evidence as a supported opaque release sidecar profile. r[mantle.release_provenance.trellis_proof_sidecars.profile]
- [x] [serial] Bind canonical envelope hash, Valence validation hash, source archive hash, release binary hash, policy hashes, proof role, claim scope, and non-claims. r[mantle.release_provenance.trellis_proof_sidecars.links]
- [x] [serial] Keep Verus source, proof IR, verifier logs, and Preserves internals opaque to Mantle core. r[mantle.release_provenance.trellis_proof_sidecars.opaque]
- [ ] [serial] Add positive fixtures for required accepted proof evidence and optional recorded-only proof evidence. r[mantle.release_provenance.trellis_proof_sidecars.positive]
  - Optional recorded-only and non-promoted formal-proof-candidate fixtures are implemented. Required accepted-proof remains blocked because Valence has not shipped the proposed Trellis accepted-proof validator or receipt vocabulary.
- [x] [serial] Add negative fixtures for missing canonical hash, stale Valence hash, source/binary mismatch, unsupported version, wrong role, unsupported scope, projection drift, malformed digest, and overclaims. r[mantle.release_provenance.trellis_proof_sidecars.negative]
- [ ] [serial] Run release-core tests and Trellis proof evidence stack smoke. r[mantle.release_provenance.trellis_proof_sidecars.validation]
  - Release-core validation is implemented locally. The stack smoke remains blocked by Valence active change `trellis-proof-evidence-profile`, whose validator, accepted-role, fixtures, graph, and final-validation tasks are all unchecked at revision `7a027529dd4b7057cf52e86dc5b258f2a9541545`.
