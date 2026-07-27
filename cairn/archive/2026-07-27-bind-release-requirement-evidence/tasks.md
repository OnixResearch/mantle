## 1. Extend the release-evidence core

- [x] [serial] 1.1 Add no-std content-bound requirement-reference and evidence-manifest types to `crunch-release-core`. r[mantle.release_provenance.content_bound_requirement_coverage]
- [x] [serial] 1.2 Add canonical BLAKE3 identities for coverage rows, evidence manifests, and release bindings. r[mantle.release_provenance.content_bound_requirement_coverage]
- [x] [serial] 1.3 Add bounded deterministic validation issues and strict relationship checks. r[mantle.release_provenance.content_bound_requirement_coverage]
- [x] [parallel] 1.4 Add unit tests for ordering, duplicate rejection, stale links, digest substitution, and weakened non-claims. r[mantle.release_provenance.content_bound_requirement_coverage]

## 2. Consume exact Cairn and Valence contracts

- [x] [serial] 2.1 Pin the exact reviewed Cairn registry schema and revision in fixtures and docs. r[mantle.release_provenance.content_bound_requirement_coverage]
- [x] [serial] 2.2 Pin the exact reviewed Valence requirement-reference schema and revision in fixtures and docs. r[mantle.release_provenance.content_bound_requirement_coverage]
- [x] [serial] 2.3 Add a selected-revision integration fixture and receipt that name exact producer revisions and schema identities. r[mantle.release_provenance.content_bound_requirement_coverage]
- [x] [parallel] 2.4 Add negative integration cases for wrong repository, wrong revision, missing requirement, stale registry, and stale receipt. r[mantle.release_provenance.content_bound_requirement_coverage]

## 3. Build evidence-manifest and provenance coverage behavior

- [x] [serial] 3.1 Represent source, test, proof, and receipt artifacts with safe paths, roles, spans or symbols, size, and BLAKE3. r[mantle.release_provenance.content_bound_evidence_manifest]
- [x] [serial] 3.2 Require producer receipt linkage for source, test, and proof evidence rows. r[mantle.release_provenance.content_bound_evidence_manifest]
- [x] [serial] 3.3 Build requirement-indexed coverage rows from typed Valence references and measured evidence rows. r[mantle.release_provenance.content_bound_requirement_coverage]
- [x] [parallel] 3.4 Reject duplicate requirement IDs, duplicate coverage roles, missing source or test coverage, unsafe paths, invalid spans, stale links, and weakened boundaries. r[mantle.release_provenance.content_bound_requirement_coverage] r[mantle.release_provenance.content_bound_evidence_manifest]

## 4. Integrate the capability-rooted release shell

- [x] [serial] 4.1 Add a no-follow capability root for requirement evidence. r[mantle.release_provenance.content_bound_evidence_manifest]
- [x] [serial] 4.2 Read bounded exact bytes and avoid ambient absolute path publication in diagnostics and manifests. r[mantle.release_provenance.content_bound_evidence_manifest]
- [x] [serial] 4.3 Add release-create flags for the evidence root and typed input file. r[mantle.release_provenance.content_bound_evidence_manifest]
- [x] [serial] 4.4 Add strict release-verify policy selection and terminal decision integration. r[mantle.release_provenance.content_bound_requirement_coverage]
- [x] [parallel] 4.5 Ensure failure does not publish a final release bundle or partial evidence manifest. r[mantle.release_provenance.content_bound_evidence_manifest]

## 5. Preserve migration compatibility

- [x] [serial] 5.1 Keep existing string coverage vectors and the traceability bridge unchanged by default. r[mantle.release_provenance.legacy_coverage_boundary]
- [x] [parallel] 5.2 Add regression tests proving absent content-bound fields preserve existing manifest serialization. r[mantle.release_provenance.legacy_coverage_boundary]
- [x] [parallel] 5.3 Prove strict policy cannot be satisfied by legacy strings or bridge comments. r[mantle.release_provenance.legacy_coverage_boundary]
- [x] [serial] 5.4 Document optional migration mode, strict mode, and future default-promotion constraints. r[mantle.release_provenance.legacy_coverage_boundary]

## 6. Preserve core purity and checked Rust

- [x] [serial] 6.1 Keep parsing, filesystem access, environment access, clocks, and process execution out of `crunch-release-core`. r[mantle.release_provenance.content_bound_requirement_coverage]
- [x] [parallel] 6.2 Run `cargo fmt`, focused tests, and strict Clippy for touched crates. r[mantle.release_provenance.content_bound_requirement_coverage] r[mantle.release_provenance.content_bound_evidence_manifest]
- [x] [parallel] 6.3 Ensure Octet pre-commit checks cover touched Rust paths. r[mantle.release_provenance.content_bound_requirement_coverage] r[mantle.release_provenance.content_bound_evidence_manifest]
- [x] [parallel] 6.4 Run no-std WASM checks for the release core. r[mantle.release_provenance.content_bound_requirement_coverage]

## 7. Add shell and CLI test coverage

- [x] [parallel] 7.1 Add positive shell tests for capability-rooted exact-byte hashing and manifest construction. r[mantle.release_provenance.content_bound_evidence_manifest]
- [x] [parallel] 7.2 Add negative shell tests for traversal, symlink escape, oversize input, malformed DTOs, stale refs, and missing receipt links. r[mantle.release_provenance.content_bound_evidence_manifest]
- [x] [parallel] 7.3 Add release CLI tests for valid strict coverage, legacy-only strict rejection, and malformed optional-present evidence. r[mantle.release_provenance.content_bound_requirement_coverage] r[mantle.release_provenance.legacy_coverage_boundary]
- [x] [parallel] 7.4 Add tamper tests proving staged bundle bytes are rehashed and relationship-validated. r[mantle.release_provenance.content_bound_evidence_manifest]

## 8. Validate integration and document boundaries

- [x] [serial] 8.1 Run the cross-repository fixture matrix against exact selected Cairn and Valence revisions. r[mantle.release_provenance.content_bound_requirement_coverage]
- [x] [parallel] 8.2 Run full workspace tests plus focused release-evidence and CLI tests. r[mantle.release_provenance.content_bound_requirement_coverage] r[mantle.release_provenance.content_bound_evidence_manifest]
- [x] [parallel] 8.3 Run focused Nix checks for added fixtures and validation rails. r[mantle.release_provenance.content_bound_evidence_manifest]
- [x] [serial] 8.4 Document ownership, rollout order, consumer policy, and non-claims. r[mantle.release_provenance.legacy_coverage_boundary]
