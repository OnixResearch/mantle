## MODIFIED Requirements

### Requirement: StageX-class self-build proof binds lineage evidence

Mantle MUST require the full StageX-class evidence tuple before this live-bootstrap source-chain change can satisfy any StageX-class bootstrap claim.
ID: bootstrap.stagex.selfbuild.proof

The proof metadata MUST bind selected provider kind, audited seed digest, lineage manifest digest, stage graph digest, normalized provider digest, staged source digest, stage1 and stage2 mantle binary digests, bootstrap-tool digests, protected execution audit digest when used, final proof bundle digest, canonical reproducibility report digest, and explicit `self-build-proof: fallback-event=<kind>` markers or `self-build-proof: fallback-event=none`. The proof MUST fail closed when any named live-bootstrap placeholder remains, when the legacy provider is selected for a StageX claim, when host-bwrap or checkout/source-discovery fallback appears, or when forbidden host executables run during the protected stage. The bootstrap parity report MUST require a checked provider-kind linkage receipt for `mantle.self-build`; the receipt MUST use the schema `crunch-self-build-provider-kind-linkage-v1` and MUST prove that `proof_identity.selected_provider_kind`, `proof_linkage.selected_provider_kind`, and `prerequisites.provider_kind` are identical closed provider-kind values. The bootstrap parity report MUST also require a checked StageX lineage provider receipt for `seed-full.stagex-lineage`; scaffold receipts MUST be explicitly marked `lineage_receipt_status = scaffold-only`, MUST serialize `provider_kind = stagex-lineage`, MUST record digest-shaped audited seed, lineage manifest, stage graph, and normalized provider fields, and MUST record no fallback events. The bootstrap parity report MUST consume real self-build proof evidence for the `crunch.self-build` row only from a validated evidence bundle that links a release manifest, `mantle-deterministic-proof-receipt-v1` deterministic proof receipt, `mantle-proof-sandbox-v1:*` sandbox evidence, release verify receipt with deterministic status `eligible`, and optional portable summary artifacts. Such evidence MAY make `crunch.self-build` evidence-backed partial for the selected provider kind, but MUST NOT satisfy Guix or StageX parity unless the selected provider kind and all axis-specific source-root or StageX lineage evidence requirements are also satisfied.

#### Scenario: StageX proof records complete evidence tuple

- GIVEN the live-bootstrap source chain materializes a normalized provider
- AND a full protected self-build proof completes with that provider
- WHEN StageX-class proof metadata is written
- THEN it records audited seed digest, lineage manifest digest, stage graph digest, normalized provider digest, staged source digest, stage1 digest, stage2 digest, bootstrap-tool digests, protected execution audit digest when used, final proof bundle digest, canonical reproducibility report digest, and explicit `self-build-proof: fallback-event=<kind>` markers or `self-build-proof: fallback-event=none`
- AND it records provider kind as StageX-class lineage serialized as `stagex-lineage`

#### Scenario: StageX proof rejects legacy fallback

- GIVEN any stage selected the legacy musl.cc provider
- WHEN StageX-class evidence is requested
- THEN the proof fails closed
- AND no StageX-class claim is emitted

#### Scenario: Release evidence binds selected provider kind [r[bootstrap.stagex.selfbuild.proof.provider-linkage]]

- GIVEN a full self-hosting proof bundle whose prerequisites name a selected provider kind
- WHEN release evidence is created from that proof bundle
- THEN `proof_linkage.selected_provider_kind` equals the proof bundle's selected provider kind
- AND verification fails if either side is missing, unknown, or mismatched

#### Scenario: Parity rejects missing provider-kind linkage receipt [r[bootstrap.stagex.selfbuild.proof.parity-missing-receipt]]

- GIVEN `bootstrap/crunch.ncl` exists
- BUT `bootstrap/evidence/crunch-self-build-provider-kind-linkage.json` is absent
- WHEN `mantle bootstrap parity-report --require guix` or `--require stagex` runs
- THEN `mantle.self-build` remains a blocker
- AND the row notes identify the missing provider-kind linkage receipt

#### Scenario: Parity accepts matching provider-kind linkage receipt as partial evidence [r[bootstrap.stagex.selfbuild.proof.parity-matching-receipt]]

- GIVEN a checked self-build provider-kind linkage receipt records the same closed provider kind in proof identity, proof linkage, and prerequisites
- WHEN the parity report loads that evidence
- THEN the `mantle.self-build` row may report evidence-backed `partial`
- AND it MUST NOT report `complete` until the full self-build proof and axis-specific evidence are present

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

#### Scenario: Parity consumes bounded real self-build proof evidence [r[bootstrap.stagex.selfbuild.proof.real-proof-parity-consumption]]

- GIVEN a checked real self-build proof evidence bundle for `crunch.self-build`
- AND the bundle links a release manifest, deterministic proof receipt, sandbox evidence, release verify receipt, and summary artifacts with matching BLAKE3 digests
- AND the deterministic proof receipt has workflow `mantle-deterministic-proof-receipt-v1`, verdict `self-rebuild-match`, selected provider kind matching the release proof linkage, two distinct clean rebuild roots, matching artifact digest sets, and sandbox profile identity `mantle-proof-sandbox-v1:*`
- AND the release verify receipt reports deterministic release status `eligible`
- WHEN the bootstrap parity report evaluates `crunch.self-build`
- THEN the row reports the real proof evidence digest and selected provider kind
- AND the row may report evidence-backed `partial` for the selected provider kind
- AND the row MUST NOT report `complete` or unblock Guix/StageX parity unless all axis-specific source-root or StageX lineage requirements are also satisfied

#### Scenario: Parity rejects malformed or unsafe real self-build proof evidence [r[bootstrap.stagex.selfbuild.proof.real-proof-fail-closed]]

- GIVEN a real self-build proof evidence bundle is missing, malformed, names an unsupported workflow version, records mismatched provider kinds, records mismatched proof or summary digests, omits supported sandbox evidence, uses a direct-host or unsupported sandbox profile, reuses a proof store/root, or has release verify status other than `eligible`
- WHEN the bootstrap parity report evaluates `crunch.self-build`
- THEN the row remains a blocker for Guix and StageX axes that require self-build proof
- AND the row notes name the specific failed evidence check
- AND no deterministic, full-source, Guix, or StageX parity claim is emitted
