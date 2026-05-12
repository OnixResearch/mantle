## MODIFIED Requirements

### Requirement: StageX-class self-build proof binds lineage evidence

Crunch MUST require the full StageX-class evidence tuple before this live-bootstrap source-chain change can satisfy any StageX-class bootstrap claim.
ID: bootstrap.stagex.selfbuild.proof

The proof metadata MUST bind selected provider kind, audited seed digest, lineage manifest digest, stage graph digest, normalized provider digest, staged source digest, stage1 and stage2 crunch binary digests, bootstrap-tool digests, protected execution audit digest when used, final proof bundle digest, canonical reproducibility report digest, and explicit `self-build-proof: fallback-event=<kind>` markers or `self-build-proof: fallback-event=none`. The proof MUST fail closed when any named live-bootstrap placeholder remains, when the legacy provider is selected for a StageX claim, when host-bwrap or checkout/source-discovery fallback appears, or when forbidden host executables run during the protected stage. The bootstrap parity report MUST require a checked provider-kind linkage receipt for `crunch.self-build`; the receipt MUST use the schema `crunch-self-build-provider-kind-linkage-v1` and MUST prove that `proof_identity.selected_provider_kind`, `proof_linkage.selected_provider_kind`, and `prerequisites.provider_kind` are identical closed provider-kind values.

#### Scenario: Release evidence binds selected provider kind [r[bootstrap.stagex.selfbuild.proof.provider-linkage]]

- GIVEN a full self-hosting proof bundle whose prerequisites name a selected provider kind
- WHEN release evidence is created from that proof bundle
- THEN `proof_linkage.selected_provider_kind` equals the proof bundle's selected provider kind
- AND verification fails if either side is missing, unknown, or mismatched

#### Scenario: Parity rejects missing provider-kind linkage receipt [r[bootstrap.stagex.selfbuild.proof.parity-missing-receipt]]

- GIVEN `bootstrap/crunch.ncl` exists
- BUT `bootstrap/evidence/crunch-self-build-provider-kind-linkage.json` is absent
- WHEN `crunch bootstrap parity-report --require guix` or `--require stagex` runs
- THEN `crunch.self-build` remains a blocker
- AND the row notes identify the missing provider-kind linkage receipt

#### Scenario: Parity accepts matching provider-kind linkage receipt as partial evidence [r[bootstrap.stagex.selfbuild.proof.parity-matching-receipt]]

- GIVEN a checked self-build provider-kind linkage receipt records the same closed provider kind in proof identity, proof linkage, and prerequisites
- WHEN the parity report loads that evidence
- THEN the `crunch.self-build` row may report evidence-backed `partial`
- AND it MUST NOT report `complete` until the full self-build proof and axis-specific evidence are present
