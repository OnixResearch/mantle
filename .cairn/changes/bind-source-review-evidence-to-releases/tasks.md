# Tasks: Bind source-review evidence to releases

## Phase 1: External contracts and baseline

- [ ] [serial] V0 Run current Cairn semantic-review validation, Mantle release-evidence tests, release-attestation tests, and focused `release_cli` verification tests before core changes. Record exact output in `cairn/changes/bind-source-review-evidence-to-releases/evidence/baseline.md`. r[verification_evidence.release_source_review_evidence]
- [ ] [serial] I1 Record exact stable Cairn producer, Valence identity, Artifact Auth signature, role, source-subject, policy, compatibility, and non-claim contracts. Do not continue from placeholder or draft-only contracts. r[verification_evidence.release_source_review_evidence]
- [ ] [serial] I2 Define typed Nickel policy and machine contracts for optional and required source-review modes, named reviewer bounds, key roles, author exclusion, revocations, and currentness. r[verification_evidence.release_source_review_evidence]

## Phase 2: Pure source-review adapter

- [ ] [serial] I3 Add typed source-review attachment and policy DTOs with BLAKE3 wrappers for release source, claim root, policy, reviewer keys, statements, Valence evidence, and attachment identity. r[verification_evidence.release_source_review_evidence]
- [ ] [serial] I4 Add pure canonicalization, exact source-link validation, external-profile validation, full-key deduplication, role separation, revocation filtering, approval counting, and deterministic reason codes. r[verification_evidence.release_source_review_evidence]
- [ ] [serial] I5 Add independent canonical-statement signature verification through the admitted Artifact Auth boundary. Producer status fields MUST remain non-authoritative. r[verification_evidence.release_source_review_evidence]
- [ ] [serial] I6 Add pure optional, satisfied, insufficient, invalid, stale, and unsupported policy outcomes without allowing review signatures to count as build witnesses. r[verification_evidence.release_source_review_evidence]

## Phase 3: Release shell and evidence

- [ ] [serial] I7 Extend release creation or attachment tooling to include an exact source-review artifact without modifying its external authority fields. r[verification_evidence.release_source_review_evidence]
- [ ] [serial] I8 Extend release verification with explicit optional and required reviewed-source policy inputs, source remeasurement, independent verification, and human/JSON summaries. r[verification_evidence.release_source_review_evidence]
- [ ] [serial] I9 Bind accepted review attachment and policy digests into release evidence and machine contracts while preserving generic release compatibility. r[verification_evidence.release_source_review_evidence]
- [ ] [serial] I10 Keep StageX no-quorum, bootstrap parity, optional build witnesses, release signatures, and source-review decisions as separate report fields and claim classes. r[verification_evidence.release_source_review_evidence]

## Phase 4: Positive and negative verification

- [ ] [parallel] V1 Add positive fixtures for optional absence, one recorded approval, and a required reviewed-source policy satisfied by the configured distinct authorized reviewer threshold. r[verification_evidence.release_source_review_evidence]
- [ ] [parallel] V2 Add negative fixtures for wrong source digest, stale claim root, wrong policy, unknown or revoked key, duplicate full key under two names, author-only approval when excluded, needs-revision disposition, insufficient approvals, bad signature, tampered Valence linkage, weakened non-claims, and unsupported schema. r[verification_evidence.release_source_review_evidence]
- [ ] [parallel] V3 Prove source-review signatures cannot satisfy release-signature or build-witness counts and that witness signatures cannot satisfy source-review policy. r[verification_evidence.release_source_review_evidence]
- [ ] [parallel] V4 Prove optional review absence does not block generic release verification, while selected reviewed-source policy fails closed before release eligibility when evidence is missing or invalid. r[verification_evidence.release_source_review_evidence]

## Phase 5: Documentation and lifecycle

- [ ] [serial] I11 Document producer/consumer workflow, exact commands, key and revocation inputs, StageX-inspired preset, optional default, role separation, evidence retention, and non-claims. r[verification_evidence.release_source_review_evidence]
- [ ] [serial] V5 Run focused source-review core tests, `nix develop -c cargo test -p crunch-release-core`, `nix develop -c cargo test -p mantle --test release_cli`, and independent external-profile compatibility checks. Record exact output in the change evidence. r[verification_evidence.release_source_review_evidence]
- [ ] [serial] V6 Run machine-contract generation and negative fixtures, `nix develop -c cargo fmt --check -p mantle -v`, focused first-party Clippy, `git diff --check`, Cairn validation, traceability coverage, all three change gates, and the relevant Nix checks. r[verification_evidence.release_source_review_evidence]
