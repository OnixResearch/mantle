# Current external producer contracts (2026-10-18)

This is a fresh owner-source inventory, not a reuse of the September blocker and not acceptance of imported proof or review authority. No builds, tests, formatters, or proofs were run during this inventory. `git rev-parse HEAD`, `b3sum`, bounded source reads, and source searches were used.

## Source identities

| Owner checkout | HEAD |
|---|---|
| Cairn | `d2c1dd7867350b787cfffe0e311fd20115238339` |
| Valence | `539d756494c3a008c12e8a6dd6121e5db92f3b34` |
| Artifact | `2a34ddd92a4eaf57d4d0de792dfa9aa4ea105121` |
| Trellis | `d91188e495d65bca87551b7a06330efb2c909edf` |

These HEAD values identify inspected checkouts, not a clean-tree or fresh upstream publication claim. Measured contract files below identify the actual inspected bytes.

## Cairn: published nearby contracts do not export a signed release-source approval

- `crates/cairn-core/src/verified/semantic_review/review_receipt.rs`, BLAKE3 `63d31a3359875996c2038edde9682718fef8c6c23a466e939e936ec730213dd6`: `validate_review_receipt` checks schema, review kind, decision, reviewer source/id metadata, reviewed-input path/hash pairs, findings, and an optional receipt hash. Reviewer metadata is not a full public key or cryptographic signature. This validator is not a source-review signature producer.
- `crates/cairn-core/src/verified/workflow_collaboration.rs`, BLAKE3 `b63c412cffaa8a600a570c9aa3d3b39aee3ae8968f8c9dfe53bfb317852551aa`: `ClaimRoot` has `claim_identity`, `subject_root`, and `revision`; `WorkflowState` carries `policy_hash`, role actor sets, evidence references, and lifecycle state; `WorkflowAction::Discharge` carries `decision` and `subject_root`. `WorkflowOperation` carries an actor and optional workflow role. These are real obligation/COB contracts, but do not specify a detached reviewer-signed release-archive statement, its canonical auth profile, or a release-source archive/projection binding.
- `cairn review validate RECEIPT --root ROOT` validates an already supplied receipt; `cairn review profiles` exports review policy metadata. Neither command creates attributable reviewer approval signatures. `crates/cairn-cli/src/semantic_review.rs` is the command owner.

Searches covered current core/CLI/docs, active changes, and accepted specifications, including `source-review`, `source_review`, `reviewed-source`, `source_subject`, `claim_root`, reviewer and discharge variants. The closest implementing APIs above were read rather than interpreting an empty semantic search as absence.

## Artifact Auth: canonical authentication exists, source-review semantics remain external

`crates/artifact-auth-core/src/lib.rs`, BLAKE3 `839ea3e35429e2a1252020dd779b46ebd185b6911674036820da118d487d1d21`, defines:

- `artifact-auth.statement.v1`, `artifact-auth.policy.v1`, and `artifact-auth.statement-preimage.v1`;
- an injective length-delimited canonical statement binding schema, domain, purpose, profile, producer/key labels, full public-key artifact reference, exact subject, sorted unique parents, and verifier context;
- Ed25519 public-key identity profile `ed25519-public-key.v1` and explicit BLAKE3/SHA-256 labels (SHA-256 is interoperability, not replacement Mantle identity);
- explicit trusted producer/key, allowed purposes, nonzero generation, currentness reference, and `Current`, `VerificationOverlap`, `Superseded`, `Revoked`, or `Unknown` currentness;
- distinct full-key threshold evaluation, with duplicate-key evidence rejected, unknown/wrong-purpose keys rejected, stale generations rejected, and only current/verification-overlap keys admitted;
- required non-claims including artifact truth, verifier correctness, authorization, revocation freshness, provenance, lifecycle, build correctness, semantic equivalence, deployment safety, and release eligibility.

`artifact-auth-ed25519/src/lib.rs`, BLAKE3 `7fd3987fe3ddc6d30efca37878952135f19905a00b52f349db237054be6b3633`, owns independent verification over core canonical bytes. It owns no signing or key lifecycle. The normal composed `verify_and_evaluate` path must not be replaced with a producer's `verified` flag.

`docs/consumer-mappings.md` maps Cairn authenticated receipts, Mantle OCI signatures, Molten currentness, and Valence protocol-verified Radicle observations. It does not declare a Cairn release-source-review subject/purpose/profile. Author exclusion, review disposition, exact claim root, source subject, and reviewed-source policy must be specified by their owning product profile; inventing those fields in Mantle would bypass the external gate.

## Valence: actual role vocabulary and actual executable roles differ

- `stack_role_registry.rs`, BLAKE3 `00dcf8c62e48bc1e45f4cdfc077a285195e2c45a8e8507c2b55737bd36f69945`, admits stack role `manual-review` and verification vocabulary `manual_review`; its `ManualReviewOnly` counting is context, never verified-property evidence. The registry explicitly proves vocabulary compatibility only.
- `evidence_ir.rs`, BLAKE3 `b8060cc7c3d00c844f5f3e29e2b95831f2c83166181f27557bd2ede3361947e6`, has the concrete `VerificationRole` enum `Property`, `RecordedOnly`, `Boundary`. `manual_review` is not an additional executable EvidenceArtifact role. A registry row alone cannot instantiate a source-review acceptance receipt.
- Current core/contracts expose no source-review identity/linkage validator that binds an exact release archive, claim root, policy, signed reviewer statements, and attachment identity. Forge-import source-review-state comments explicitly do not become Radicle approvals.

## Exact remaining reviewed-source prerequisite

Cairn must publish a versioned, independently verifiable source-review export: exact archive subject/profile and immutable revision/tree linkage; claim root/current revision; review-policy identity; approval versus needs-revision semantics; canonical Artifact Auth domain/purpose/profile and signed statement/parent/context mapping; full reviewer-key identities; admitted reviewer roles; explicit currentness/revocation input contract; author-exclusion semantics; bounded non-claims and compatibility vectors. Valence must publish the corresponding identity/linkage acceptance contract, accepted non-property manual-review role semantics, artifact/receipt identities, and positive/negative compatibility vectors. Publication must identify an immutable reviewed source revision.

No real source-review producer command can be supplied from these current APIs. No approval fixture, synthetic signature, new Mantle placeholder DTO, or producer status string was substituted. Therefore review tasks I1-I10 and observation integration I3 are not complete. Source-built compatibility I4 remains independently open until the actual source-built proof owner stabilizes and exercises that boundary.

## Reachable Mantle hardening

The existing seven-field `SourceObservationBinding` is kept. Its digest is now reconstructed by `SourceObservationBinding::for_acquisition` using the actual source core, exact measured archive bytes, immutable Git revision/object format where applicable, release profile `mantle-release-source` version `v1` mapped to numeric observation profile `1`, and fixed logical archive projection `source`. Arbitrary distinct hashes are no longer accepted as observations. Mutable refs and mirror locations remain outside the observation identity. The enclosing release manifest/signature binds these facts, with no additional signer or reviewer count.

Generic release absence remains optional and unchanged. Supplying a binding without acquisition metadata fails instead of dropping evidence. Import preflights all records, intra-bundle storage-identity collisions, and an existing pin before writing. Identical records/pins are write-free. Record/pin publication uses create-new persistence, not replacing rename. This is not a transactional multi-file import guarantee: process interruption or I/O failure after some successful additions can leave those additions; no all-or-nothing crash claim is made.

## Preparation and verification commands

These are commands to execute after central integration, not claimed executions here:

```sh
nix run path:../cairn#cairn -- review profiles
nix run path:../cairn#cairn -- review validate "$EXISTING_REVIEW_RECEIPT" --root "$REVIEW_ROOT"
nix develop -c cargo test -p crunch-release-core observation
nix develop -c cargo test -p mantle --lib source_observation::tests::
nix develop -c cargo test -p mantle --lib source_bundle::readback_tests::
nix develop -c cargo test -p mantle --lib release_evidence::tests::create_and_verify_release_bundle_records_source_acquisition_url
nix develop -c cargo test -p mantle --lib release_evidence::tests::orphan_source_observation
```

The first two commands do not create signed review authority. Real source-observation preparation for `release create --source-observation` is described in ADR 0059. Required review association tests cannot honestly run before the missing producer/Valence contract exists.
