# Tasks: import Nario v2 store archives

## Phase 1: Format authority and pure core

- [ ] [serial] I1 Pin one Determinate Nix source revision and exact Nario v2 producer version. Retain the source identity and format references. r[store_transports.nario_v2_read_compatibility]
- [ ] [serial] I2 Generate durable positive and negative Nario v2 fixtures with the pinned producer. r[store_transports.nario_v2_validation]
- [ ] [serial] I3 Add a pure bounded framing, metadata, record-state, limit, and diagnostic core for Nario v2 reads. r[store_transports.nario_v2_bounded_admission]
- [ ] [parallel] I4 Add core tests for valid records, unsupported features, malformed order, duplicate paths, truncation, oversized fields, and limit exhaustion. r[store_transports.nario_v2_bounded_admission]

## Phase 2: Store list and import

- [ ] [serial] I5 Add explicit `nario-v2` format selection to store archive list and import without changing the Mantle-native default. r[store_transports.nario_v2_read_compatibility]
- [ ] [serial] I6 Stage supported Nario records, validate the complete archive, then publish PathInfo through one rollback-capable mutation boundary. r[store_transports.nario_v2_bounded_admission]
- [ ] [parallel] I7 Keep staging, import, rollback, and existing-path skips bounded for seekable and non-seekable input. r[store_transports.nario_v2_bounded_admission]
- [ ] [parallel] I8 Reject unsupported Nario metadata, wrong prefixes, untrusted signatures, path conflicts, stale content, and incomplete archives without a success receipt. r[store_transports.nario_v2_validation]

## Phase 3: Foreign source preparation

- [ ] [serial] I9 Allow source preparation to match exact executable-plan source requirements against verified Nario records. r[foreign_derivation_import.nario_v2_source_preparation]
- [ ] [serial] I10 Decode and canonicalize matched NAR payloads into target source records with separate original and target identities. r[foreign_derivation_import.nario_v2_source_preparation]
- [ ] [parallel] I11 Reject arbitrary built outputs, unmatched paths, unsafe trees, wrong hashes, wrong modes, and ambiguous record matches. r[foreign_derivation_import.nario_v2_source_preparation]
- [ ] [parallel] I12 Bind archive, original PathInfo, target source, plan, policy, and requirement identities into source preparation evidence. r[foreign_derivation_import.nario_v2_non_claims]

## Phase 4: Compatibility and claim boundaries

- [ ] [serial] I13 Prove list and import behavior against fresh archives from the pinned Determinate Nix producer. r[store_transports.nario_v2_validation]
- [ ] [parallel] I14 Document supported direction, exact format version, unsupported metadata, trust policy, source projection, and Nario export as unsupported. r[foreign_derivation_import.nario_v2_non_claims]
- [ ] [parallel] I15 State that Nario carries store data, not package recipes, Nixpkgs selection meaning, evaluator parity, correctness, or reproducibility. r[foreign_derivation_import.nario_v2_non_claims]

## Phase 5: Verification and lifecycle

- [ ] [serial] V1 Run focused framing, NAR, archive, PathInfo, trust, CA, source preparation, and CLI tests. Record exact output in `evidence/verification.md`. r[store_transports.nario_v2_validation]
- [ ] [serial] V2 Run formatting, focused Clippy, workspace checks, compatibility fixture generation, `git diff --check`, and machine-contract checks. r[store_transports.nario_v2_validation]
- [ ] [serial] V3 Run Cairn validation, proposal, design, and tasks gates, Tracey coverage, sync, archive, and the relevant Nix checks. r[store_transports.nario_v2_validation]
