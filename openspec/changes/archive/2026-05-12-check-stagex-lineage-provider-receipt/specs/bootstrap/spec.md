## MODIFIED Requirements

### Requirement: StageX-class self-build proof binds lineage evidence

Crunch MUST require the full StageX-class evidence tuple before this live-bootstrap source-chain change can satisfy any StageX-class bootstrap claim.
ID: bootstrap.stagex.selfbuild.proof

The proof metadata MUST bind selected provider kind, audited seed digest, lineage manifest digest, stage graph digest, normalized provider digest, staged source digest, stage1 and stage2 crunch binary digests, bootstrap-tool digests, protected execution audit digest when used, final proof bundle digest, canonical reproducibility report digest, and explicit `self-build-proof: fallback-event=<kind>` markers or `self-build-proof: fallback-event=none`. The proof MUST fail closed when any named live-bootstrap placeholder remains, when the legacy provider is selected for a StageX claim, when host-bwrap or checkout/source-discovery fallback appears, or when forbidden host executables run during the protected stage. The bootstrap parity report MUST require a checked provider-kind linkage receipt for `crunch.self-build`; the receipt MUST use the schema `crunch-self-build-provider-kind-linkage-v1` and MUST prove that `proof_identity.selected_provider_kind`, `proof_linkage.selected_provider_kind`, and `prerequisites.provider_kind` are identical closed provider-kind values. The bootstrap parity report MUST also require a checked StageX lineage provider receipt for `seed-full.stagex-lineage`; scaffold receipts MUST be explicitly marked `lineage_receipt_status = scaffold-only`, MUST serialize `provider_kind = stagex-lineage`, MUST record digest-shaped audited seed, lineage manifest, stage graph, and normalized provider fields, and MUST record no fallback events.

#### Scenario: Parity rejects missing StageX lineage receipt [r[bootstrap.stagex.selfbuild.proof.stagex-lineage-missing-receipt]]

- GIVEN no StageX lineage provider receipt exists
- WHEN the bootstrap parity report evaluates `seed-full.stagex-lineage`
- THEN the row remains a StageX blocker
- AND the row notes identify the missing lineage receipt

#### Scenario: Parity accepts scaffold lineage receipt only as partial evidence [r[bootstrap.stagex.selfbuild.proof.stagex-lineage-scaffold-partial]]

- GIVEN a StageX lineage provider receipt has schema `crunch-stagex-lineage-provider-receipt-v1`
- AND it records `provider_kind = stagex-lineage`, `lineage_receipt_status = scaffold-only`, digest-shaped lineage fields, and no fallback events
- WHEN the bootstrap parity report evaluates `seed-full.stagex-lineage`
- THEN the row may report evidence-backed `partial`
- AND it MUST NOT report `complete` or unblock StageX parity until real audited lineage and self-build proof evidence exist
