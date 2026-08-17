## Tasks

- [x] [serial] Extend function-address release evidence metadata to include canonical Kamacite Preserves receipt sidecars. r[mantle.release_provenance.function_address_preserves_sidecars.contract]
- [x] [serial] Keep Preserves sidecar parsing out of Mantle release validation core. r[mantle.release_provenance.function_address_preserves_sidecars.opaque]
- [x] [serial] Bind JSON compatibility sidecars to canonical Preserves receipt identity when present. r[mantle.release_provenance.function_address_preserves_sidecars.json_projection]
- [x] [serial] Add positive fixtures for optional and required Preserves-backed release binding. r[mantle.release_provenance.function_address_preserves_sidecars.positive]
- [x] [serial] Add negative fixtures for missing/stale hashes, projection drift, wrong role/schema, wrong claim scope, malformed BLAKE3, source/binary mismatch, and overclaims. r[mantle.release_provenance.function_address_preserves_sidecars.negative]
- [x] [serial] Run release-core tests and the Preserves-backed stack smoke. r[mantle.release_provenance.function_address_preserves_sidecars.validation]

## Evidence summary

- Canonical Preserves bytes are bound by BLAKE3 and size without parsing their payload.
- Full Valence and JSON compatibility envelopes preserve public logical identities for direct Cairn consumption.
- Core, CLI, machine-contract, wasm, Clippy, formatting, and focused Tiger Style evidence is recorded in `evidence/validation.md`.
