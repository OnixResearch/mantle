## ADDED Requirements

### Requirement: Mantle defines portable build receipt bundles [r[portable_build_receipts.receipt_bundle_format]]

Mantle MUST define a versioned portable receipt bundle format for remote/offline build evidence. The format MUST bind policy hash, logical store prefix when outputs are named, action refs, output identities, receipt/attestation refs, semantic graph records, trust-basis summaries, deterministic ordering, BLAKE3 record digests, and named limits for variable-length fields. Unsupported versions, malformed records, path traversal, duplicate conflicting records, or unsupported mandatory record kinds MUST fail closed.

#### Scenario: Equivalent receipt bundle canonicalizes deterministically [r[portable_build_receipts.receipt_bundle_format.scenario.deterministic]]

- GIVEN equivalent receipt bundle records are gathered in different traversal orders
- WHEN Mantle canonicalizes the portable receipt bundle
- THEN the bundle digest and record order MUST be identical
- AND host temp paths, unbounded logs, and serialization map order MUST NOT affect bundle identity.

#### Scenario: Malformed receipt bundle is rejected [r[portable_build_receipts.receipt_bundle_format.scenario.reject-malformed]]

- GIVEN a receipt bundle has an unsupported version, duplicate conflicting record, path traversal in an embedded sidecar name, oversized metadata, wrong store prefix, or unsupported mandatory record kind
- WHEN Mantle parses or verifies the bundle
- THEN Mantle MUST reject the bundle with deterministic diagnostics
- AND it MUST NOT persist receipt, attestation, or graph material from that bundle.

### Requirement: Receipt bundles bind trust snapshots and revocation state [r[portable_build_receipts.trust_snapshot_revocation]]

Mantle MUST bind the trust context used for portable receipt verification. Receipt bundles and verification reports MUST identify public key material by digest, policy hash, trust-root snapshot digest when replayed, revocation-list refs when applicable, and expiration windows. Verification MUST reject revoked, expired, same-name-different-key, or policy-mismatched trust material before importing evidence or making a trust claim.

#### Scenario: Trust snapshot verifies replay [r[portable_build_receipts.trust_snapshot_revocation.scenario.snapshot]]

- GIVEN a portable receipt bundle includes a trust-root snapshot digest, trusted public key identities, policy hash, revocation-list ref, and verification time window
- WHEN Mantle verifies the bundle in replay mode
- THEN Mantle MUST evaluate signatures and trust policy against that explicit trust snapshot
- AND the verification report MUST identify the snapshot and public key digests used.

#### Scenario: Revoked or expired signer is rejected [r[portable_build_receipts.trust_snapshot_revocation.scenario.reject-revoked]]

- GIVEN a bundle's signer key is revoked, expired for the requested verification window, or has the same signer name but different key material than the trusted key
- WHEN Mantle verifies the portable receipt bundle
- THEN Mantle MUST reject the trust basis with deterministic diagnostics
- AND it MUST NOT import the bundle as trusted evidence.

### Requirement: Receipt bundles classify evidence-chain completeness [r[portable_build_receipts.evidence_chain_completeness]]

Mantle MUST classify whether a portable receipt bundle contains a complete evidence chain for the requested claim strength. Strong action-correctness or release-facing claims MUST require matching source/input refs, action refs, sandbox policy evidence, network policy evidence, reference-scan evidence, output object refs, artifact or closure attestations, producer policy, and required signatures. Missing chain elements MUST produce a narrower diagnostic classification rather than a strong claim.

#### Scenario: Complete chain supports strong claim [r[portable_build_receipts.evidence_chain_completeness.scenario.complete]]

- GIVEN a bundle contains matching source refs, action refs, sandbox and network policy evidence, reference-scan evidence, output object refs, attestations, producer policy, and trusted signatures
- WHEN Mantle verifies the bundle for a strong action-correctness claim
- THEN Mantle MAY classify the evidence chain as complete for that bounded output
- AND the report MUST list the matched evidence classes.

#### Scenario: Partial chain stays diagnostic [r[portable_build_receipts.evidence_chain_completeness.scenario.partial]]

- GIVEN a bundle has output signatures and PathInfo but lacks source refs, sandbox reports, reference scans, action refs, or required attestations for the requested strong claim
- WHEN Mantle verifies the bundle
- THEN Mantle MUST classify the bundle as partial or diagnostic evidence only
- AND it MUST NOT report strong action-correctness or release-facing readiness.

### Requirement: Mantle exports and lists available receipt evidence without fabrication [r[portable_build_receipts.receipt_bundle_export_list]]

Mantle MUST export and list only receipt, attestation, graph, and trust-basis material that exists or is reconstructed by documented deterministic rules. Export/list reports MUST identify missing evidence classes rather than fabricating action refs, sandbox reports, source refs, graph edges, or attestations.

#### Scenario: Export gathers available evidence [r[portable_build_receipts.receipt_bundle_export_list.scenario.export]]

- GIVEN a local output has PathInfo, artifact attestation, action receipt, source ref, sandbox report, and semantic graph records available
- WHEN Mantle exports a portable receipt bundle for that output
- THEN the bundle MUST include those evidence records with deterministic digests
- AND the export report MUST identify the output identities the bundle explains.

#### Scenario: Missing evidence is reported narrowly [r[portable_build_receipts.receipt_bundle_export_list.scenario.missing]]

- GIVEN a local output is missing action receipt, sandbox report, source ref, attestation, or semantic graph material required for a strong claim
- WHEN Mantle exports or lists a receipt bundle
- THEN Mantle MUST report the missing evidence class
- AND it MUST NOT invent placeholder evidence or claim strong build correctness from incomplete material.

### Requirement: Mantle verifies portable receipt bundles against output facts [r[portable_build_receipts.receipt_bundle_verify]]

Mantle MUST verify portable receipt bundles against local output facts, store archive metadata, or remote-build output metadata before treating them as evidence. Verification MUST check output identity, action ref when present, object refs, PathInfo signatures, store prefix, policy hash, source refs, sandbox/network reports for strong claims, and configured trust roots.

#### Scenario: Matching bundle verifies for a copied output [r[portable_build_receipts.receipt_bundle_verify.scenario.match]]

- GIVEN a copied store output or store archive record has output facts matching a portable receipt bundle
- AND the bundle's signatures, object refs, action ref, store prefix, and policy hash match the requested trust policy
- WHEN Mantle verifies the bundle against those output facts
- THEN verification MUST succeed for that bounded output identity
- AND the verification report MUST identify the matched receipt and trust-basis records.

#### Scenario: Stale or untrusted evidence fails [r[portable_build_receipts.receipt_bundle_verify.scenario.reject-stale]]

- GIVEN a receipt bundle has a stale action ref, mismatched output object ref, wrong store prefix, missing required sandbox report, unknown signer, same-name-different-key signer, or policy mismatch
- WHEN Mantle verifies the bundle
- THEN Mantle MUST reject the bundle for the requested evidence strength
- AND it MUST NOT mark the output as receipt-verified.

### Requirement: Mantle imports receipt bundles idempotently and preserves graph integrity [r[portable_build_receipts.receipt_bundle_import]]

Mantle MUST import only verified portable receipt bundle material. Import MUST be idempotent for equivalent records, reject conflicting records, preserve artifact/closure attestation identities, and add semantic graph nodes and edges only when their immutable identities and referenced evidence match local or imported output facts.

#### Scenario: Verified bundle imports graph evidence [r[portable_build_receipts.receipt_bundle_import.scenario.import]]

- GIVEN a portable receipt bundle has verified against local or archive-provided output facts
- WHEN Mantle imports the bundle
- THEN Mantle MUST persist the verified receipt, attestation, and semantic graph records
- AND subsequent graph queries MAY explain the remote/offline output using those imported immutable identities.

#### Scenario: Conflicting graph import fails closed [r[portable_build_receipts.receipt_bundle_import.scenario.reject-conflict]]

- GIVEN local state already contains a receipt, attestation, graph node, or graph edge with the same identity but different bytes or incompatible referenced evidence
- WHEN Mantle imports a portable receipt bundle
- THEN Mantle MUST reject the conflicting record or the whole import according to documented policy
- AND it MUST NOT overwrite local evidence silently.

#### Scenario: Missing graph evidence remains incomplete [r[portable_build_receipts.receipt_bundle_import.scenario.incomplete-graph]]

- GIVEN an imported output lacks verified graph evidence for its producing action or source inputs
- WHEN an operator queries why or graph information for that output
- THEN Mantle MUST return an incomplete semantic graph diagnostic
- AND it MUST NOT invent source, recipe, proof, or witness edges from output bytes alone.

### Requirement: Portable receipt evidence stays bounded [r[portable_build_receipts.receipt_bundle_non_claims]]

Mantle MUST keep portable receipt bundle claims bounded to the verified evidence present in the bundle and matched output facts. Receipt bundle verification MUST NOT by itself claim compiler correctness, full source-to-binary reproducibility, deploy success, frontend module correctness, remote builder honesty beyond accepted signatures, or output payload import unless separate evidence proves those claims.

#### Scenario: Receipt bundle proof does not overclaim [r[portable_build_receipts.receipt_bundle_non_claims.scenario.non-claim]]

- GIVEN Mantle verifies or imports a portable receipt bundle for a remote/offline handoff
- WHEN a task, evidence file, status reply, graph query, or build report cites that result
- THEN the claim MUST be limited to the matched receipt, attestation, graph, and trust-basis evidence
- AND it MUST NOT claim build execution success, payload transfer, compiler correctness, full reproducibility, deploy success, or frontend module correctness without separate evidence.
