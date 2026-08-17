## Phase 1: Source and mapping

- [x] [serial] I1 Record the baseline, portfolio frame, exact RID, reviewed revision, producer archive, and consumer boundary. r[mantle.durable_file_publication.source]
- [x] [serial] I2 Add aligned Cargo and Nix pins with package, revision, license, and no-fallback assertions. r[mantle.durable_file_publication.source]
- [x] [serial] I3 Add the pure Mantle request and disposition mapping for immutable remote-attempt objects. r[mantle.durable_file_publication.mapping] r[mantle.durable_file_publication.outcomes]
- [x] [depends:mantle.durable_file_publication.mapping] I4 Route the production immutable segment and anchor shell through one caller-opened parent capability. r[mantle.durable_file_publication.adapter]
- [x] [parallel] I5 Keep manifest replacement, existing-content comparison, retention, deletion, receipts, and diagnostics in Mantle. r[mantle.durable_file_publication.authority]
- [x] [parallel] I6 Preserve the existing publisher as an explicit rollback backend without automatic fallback. r[mantle.durable_file_publication.authority]

## Phase 2: Validation and evidence

- [x] [parallel] V1 Replay all 17 shared corpus rows and test positive plus negative Mantle mappings. r[mantle.durable_file_publication.validation]
- [x] [parallel] V2 Run Linux filesystem tests for success, idempotence, conflicts, destination races, symlinks, bounds, cleanup, and committed durability uncertainty. r[mantle.durable_file_publication.validation]
- [x] [serial] V3 Emit typed Nickel and deterministic JSON/BLAKE3 adoption evidence with explicit non-claims. r[mantle.durable_file_publication.evidence]
- [x] [serial] V4 Run focused Cargo, formatting, Clippy, Tiger Style, Nickel, Nix, Cairn, traceability, and repository validation. r[mantle.durable_file_publication.validation]
- [x] [serial] V5 Synchronize the accepted specification and archive only after the implementation and evidence pass. r[mantle.durable_file_publication.evidence]
