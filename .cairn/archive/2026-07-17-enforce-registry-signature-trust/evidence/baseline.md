# Baseline and portfolio review

## Success contract

Goal: authenticate the exact immutable OCI image/metadata manifest pair under explicit operator policy before registry pull can download its content closure or reach local admission.

Observable completion evidence:

- push publishes and immutably re-verifies a deterministic signature artifact;
- pull requires its immutable digest and verifies policy/signatures before content download/admission;
- trusted round trip succeeds through ordinary OCI import;
- malformed, unknown, revoked, wrong-domain/repository, drifted, duplicated, and bad-signature cases fail without output/receipt/admission;
- receipt/schema/docs/lifecycle rails agree on narrow claims.

False-completion cases: bearer authentication only; signer-name matching without cryptographic verification; signing only one of the two manifest digests; verification after import; local receipt-only signatures unavailable to pull; tag equality treated as immutability; registry possession treated as trust; duplicate signatures inflating quorum; credentials/key paths entering identity; or successful receipts surviving failed trust.

Audit risks: key-name collision, revoked-key selection, cross-domain/repository replay, signature/artifact substitution, algorithm downgrade, mutable tags, TOCTOU between tag and digest reads, unbounded policy/signature collections, secret leakage, and unsupported authorization/transparency claims.

Budget: one existing-code search, focused reads of registry core/shell/CLI/signing infrastructure, three materially different design mechanisms, one adversarial audit, focused deterministic tests, and repository lifecycle/quality rails. Accepted terminal outcomes are validated, an exact bounded blocker, exhausted, or user-decision-required.

## Approach registry

### Detached OCI signature artifact — selected

- **Mechanism:** third subject-bound OCI artifact containing a deterministic detached-signature document over image+metadata digests and trust domain.
- **Claim:** remote immutable signature evidence can be fetched and verified before content admission without changing existing image/metadata bytes.
- **Artifact:** ADR 0029, `src/oci_registry.rs`, and registry shell/CLI fixtures.
- **Gap strength:** equivalent to the bounded goal.
- **Next check:** prove signature verification precedes content blob GETs and output creation in adversarial CLI tests.
- **State:** active.

### Embed signatures into companion metadata — rejected

- **Mechanism:** add signature bytes to the metadata artifact itself.
- **Claim:** two manifests remain sufficient.
- **Known failure:** signing the metadata-manifest digest is circular if the signature is inside that manifest; signing only the image or metadata payload does not authenticate the final immutable pair.
- **State:** falsified.

### Sign only the local push receipt — rejected

- **Mechanism:** detached local receipt signature with no registry artifact.
- **Claim:** operators can authenticate a handoff file out of band.
- **Known failure:** registry pull has no bounded remote authority unless another transport is introduced, and receipt availability/identity becomes an undocumented prerequisite.
- **State:** blocked as weaker than the operator-facing registry goal.

## Adversarial audit of selected mechanism

The signed statement excludes registry routing to preserve mirror/content separation; repository replay is constrained by the verifier's explicit policy. The statement domain-separates schema/suite/trust-domain and both immutable digests. Pull must resolve all tags against operator-supplied digests, verify signature manifest linkage and signature blob descriptor, then enforce full-key trust/revocation/threshold before downloading image or metadata content blobs. Signer names remain labels only; verified public-key BLAKE3 identities establish distinct quorum members.

Residual non-claims: no registry authorization proof, transparency log, revocation freshness, ambient trust discovery, tag immutability, arbitrary registry compatibility, artifact correctness, kernel compatibility, bootability, deployability, or release eligibility.

## Current deterministic baseline

Pueue task `79` ran before core changes:

```text
nix develop -c cargo test -p mantle --lib oci_registry
nix develop -c cargo test -p mantle --test kernel_bundle_oci_registry_cli
```

Result:

```text
running 7 tests
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 109 filtered out
running 4 tests
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
