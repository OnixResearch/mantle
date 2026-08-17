## Tasks

- [x] [serial] Register Trellis proof evidence as a supported opaque release sidecar profile. r[mantle.release_provenance.trellis_proof_sidecars.profile]
- [x] [serial] Bind canonical envelope hash, Valence validation hash, source archive hash, release binary hash, policy hashes, proof role, claim scope, and non-claims. r[mantle.release_provenance.trellis_proof_sidecars.links]
- [x] [serial] Keep Verus source, proof IR, verifier logs, and Preserves internals opaque to Mantle core. r[mantle.release_provenance.trellis_proof_sidecars.opaque]
- [x] [serial] Add positive fixtures for required accepted proof evidence and optional recorded-only proof evidence. r[mantle.release_provenance.trellis_proof_sidecars.positive]
  - Optional recorded-only, non-promoted candidate, and exact candidate-plus-property required fixtures pass; unauthorized role pairs fail closed.
- [x] [serial] Add negative fixtures for missing canonical hash, stale Valence hash, source/binary mismatch, unsupported version, wrong role, unsupported scope, projection drift, malformed digest, and overclaims. r[mantle.release_provenance.trellis_proof_sidecars.negative]
- [x] [serial] Run release-core tests and Trellis proof evidence stack smoke. r[mantle.release_provenance.trellis_proof_sidecars.validation]
  - Valence commit `27b8b212` focused accepted/negative profile tests pass; Mantle focused Trellis tests pass 7/7, full release-core passes 207/207, and strict core Clippy plus formatting pass.
