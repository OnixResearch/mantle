## Phase 1: Generic projection and admission

- [x] [depends:onixos.define-onix-kernel-bundle-v1] Capture the reviewed frontend projection/response fixtures and keep all Onix semantic validation in the supplied attestation. r[kernel_bundle_oci.projection]
- [x] [serial] Define `mantle-oci-projection-v1`, typed digest roles, layer/archive policies, named bounds, reconstruction response, and non-claims as frontend-neutral DTOs. r[kernel_bundle_oci.projection] r[kernel_bundle_oci.digest_roles]
- [x] [serial] Implement pure projection, attestation, object-ref, ordering, media/annotation, digest-role, and no-hidden-fallback admission. r[kernel_bundle_oci.admission]

## Phase 2: Deterministic export

- [x] [serial] Implement pure canonical file/directory layer plans with checked size/depth/entry arithmetic, normalized metadata, symlink rules, and deterministic compression profiles. r[kernel_bundle_oci.layering]
- [x] [serial] Implement the thin CAS/archive/OCI-layout shell with staging, complete verification, atomic publication, and no ambient Nix or host-path fallback. r[kernel_bundle_oci.export]
- [x] [serial] Preserve frontend media types and annotations exactly while computing OCI SHA-256 and Mantle BLAKE3 in separate typed fields. r[kernel_bundle_oci.digest_roles] r[kernel_bundle_oci.export]

## Phase 3: Verified import and reports

- [x] [serial] Implement pure OCI layout/descriptor graph validation and import classification before any CAS commit. r[kernel_bundle_oci.import]
- [x] [serial] Implement the thin descriptor-first import shell, exact blob verification, atomic CAS admission, rollback/unreferenced-partial handling, and reconstruction response. r[kernel_bundle_oci.import]
- [x] [depends:mantle.expand-machine-artifact-contract-registry] Register export/import reports with Rust DTO ownership, exact schemas, generated Nickel contracts, version policy, BLAKE3 freshness, and non-claims. r[kernel_bundle_oci.reports]

## Phase 4: Positive and negative evidence

- [x] [parallel] Add positive fixtures for exact file layers, canonical directory layers, KBI-compatible media/annotations, Onix round-trip reconstruction, and external compatibility-only import. r[kernel_bundle_oci.verification]
- [x] [parallel] Add negative fixtures for missing admission, stale spec/projection/object identity, wrong digest role, malformed descriptor graph, blob size/hash mismatch, path escape, special file, symlink escape, duplicate descriptor, oversized layout, partial import, and Onix overclaim. r[kernel_bundle_oci.verification]
- [x] [parallel] Add determinism fixtures proving declaration/traversal order and ambient ownership/time metadata do not change canonical Mantle layer bytes. r[kernel_bundle_oci.layering] r[kernel_bundle_oci.verification]

## Phase 5: Documentation and closeout

- [x] [parallel] Document CLI use, local-layout scope, canonical archive profile, Onix/Mantle ownership, digest roles, KBI preservation, external import states, and registry non-goals. r[kernel_bundle_oci.reports]
- [x] [serial] Run focused pure-core, artifact CLI, CAS, schema/contract, round-trip, dependency-audit, formatting, clippy, and first-party tests. r[kernel_bundle_oci.verification]
- [ ] [serial] Run Cairn validation and proposal/design/tasks gates; sync and archive only after cross-repo round-trip and positive/negative evidence are recorded. r[kernel_bundle_oci.verification]
