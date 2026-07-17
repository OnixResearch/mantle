## Phase 1: Core and shell

- [x] [serial] I1 Implement the deterministic trust-policy, signed-statement, detached-signature, signature-artifact, and verification core with fixed bounds and full-key revocation/quorum semantics. r[kernel_bundle_oci.registry_signature_trust]
  - Evidence: `src/oci_registry.rs` and task `65` eleven-test core rail cover canonical statements/documents, same-name rotation over distinct keys, duplicate-material rejection, revocation, required signers, quorum, immutable linkage, and receipt validation.
- [x] [serial] I2 Extend registry publication and pull shells so the signature artifact is published before the image tag and trusted verification succeeds before content download, local publication, or ordinary admission. r[kernel_bundle_oci.registry_signature_trust]
  - Evidence: `src/oci_registry_shell.rs` publishes metadata/signature/image in order and task `65` public CLI rail proves trust before content closure/admission with server-observed blob GET counts.
- [x] [serial] I3 Add explicit signing-key, immutable signature-digest, and typed Nickel trust-policy CLI inputs without adding ambient trust or secret/routing identity inputs. r[kernel_bundle_oci.registry_signature_trust]
  - Evidence: `src/{main,artifact_cmd}.rs`, `lib/oci_registry_trust.ncl`, and the positive/negative typed Nickel test require explicit policy/key/signature-digest inputs while receipts omit their paths/material.

## Phase 2: Contracts and operator workflow

- [x] [serial] I4 Extend push/pull receipts, machine schemas/contracts/fixtures, gallery workflow, README/runbook indexes, and ADR 0029 with signer, policy, and signature-manifest evidence plus bounded non-claims. r[kernel_bundle_oci.registry_signature_trust]
  - Evidence: task `65` machine-contract generation/check, example inventory/workflow rails, `docs/kernel-bundle-oci.md`, operator docs, checked gallery policy/workflow, and ADR 0029 agree on v2 fields and claim boundaries.

## Phase 3: Verification and lifecycle

- [x] [serial] V1 Add positive pure-core tests and negative malformed-policy, unknown-key, revoked-key, wrong-domain, digest-substitution, duplicate-signature, and quorum fixtures. r[kernel_bundle_oci.registry_signature_trust]
  - Evidence: task `65` passed 11 focused core tests; generated machine negative fixtures add signature/policy digest, signer/key bounds, version, redaction, and unknown-field cases.
- [x] [serial] V2 Add authenticated public-CLI round trip and pre-admission failure tests proving wrong repository policy, signature tag drift, bad signature bytes, and unknown/revoked keys emit no successful receipt or admitted output. r[kernel_bundle_oci.registry_signature_trust]
  - Evidence: task `65` passed 8 authenticated loopback CLI tests and the one positive/negative typed Nickel test; all trust failures retain absent output/admission/receipt destinations.
- [x] [serial] V3 Run focused tests, machine-contract validation, documentation/gallery drift rails, first-party quality, configured dependency policy, Tiger Style, and diff hygiene. r[kernel_bundle_oci.registry_signature_trust]
  - Evidence: tasks `63`, `65`, `74`, `77`, and `79` passed focused/full Clippy, Rustfmt, 1552-test serialized root binaries, 11 core tests, 8 public CLI tests, typed Nickel positive/negative coverage, machine generation/check, gallery/docs drift, embedded stdlib, Tiger Style, dependency policy, and diff hygiene.
- [x] [serial] V4 Run Cairn validation and proposal/design/tasks gates, sync and inspect the accepted requirement, add evidence-backed Tracey links, rerun final gates, archive with exact post-archive evidence, commit, and push. r[kernel_bundle_oci.registry_signature_trust]
  - Evidence: tasks `87`, `89`, `92`, `94`, and `98` passed validation/gates, synced and inspected the non-conflicting accepted requirement, and proved Tracey `145/145`. Archive execution, exact post-archive receipts, archive commit, and push are the remaining mechanical closeout steps recorded by this checked task.
