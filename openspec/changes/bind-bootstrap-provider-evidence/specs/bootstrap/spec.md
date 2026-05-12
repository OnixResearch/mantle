## MODIFIED Requirements

### Requirement: Full-source bootstrap claim requires evidence

Crunch MUST withhold the full-source bootstrap claim until the source-root manifest validates for the full-source profile, the lineage manifest validates for the StageX-class profile, every named live-bootstrap placeholder is replaced, source-built stage transcripts exist, and self-build proof completes with the selected source-built provider.
ID: bootstrap.fullsource.claim.evidence

The claim evidence MUST include provider kind (`source-root` for the full-source profile or StageX-class lineage serialized as `stagex-lineage` for the StageX-class profile), manifest digest, provider output digest, proof bundle digest, stage-by-stage build transcripts through `bootstrap/seed-full.ncl`, `bootstrap/selftest.ncl` and `bootstrap/integration-test.ncl` transcripts, proof metadata bound to the selected source-built provider, explicit `self-build-proof: fallback-event=<kind>` markers or `self-build-proof: fallback-event=none`, and a docs update that separates remaining trusted roots from eliminated binary-provider trust. A prerequisite-only check, placeholder derivation, deferred task, archived partial-scaffolding change, unfinished successor task, legacy-provider fallback, host-bwrap fallback, or checkout/source-discovery fallback MUST NOT count as full-source bootstrap evidence. The normalized `bootstrap/seed-full.ncl` provider metadata MUST serialize its provider kind as `source-root`; StageX evidence MUST serialize the selected provider kind as `stagex-lineage` and MUST NOT be inferred from source-root metadata alone.

#### Scenario: Seed-full declares source-root provider kind

- GIVEN `bootstrap/seed-full.ncl` builds its provider metadata
- WHEN the metadata is inspected
- THEN it contains `provider_kind = source-root`
- AND the metadata does not claim `stagex-lineage`

### Requirement: StageX-class self-build proof binds lineage evidence

Crunch MUST require the full StageX-class evidence tuple before this live-bootstrap source-chain change can satisfy any StageX-class bootstrap claim.
ID: bootstrap.stagex.selfbuild.proof

The proof metadata MUST bind selected provider kind, audited seed digest, lineage manifest digest, stage graph digest, normalized provider digest, staged source digest, stage1 and stage2 crunch binary digests, bootstrap-tool digests, protected execution audit digest when used, final proof bundle digest, canonical reproducibility report digest, and explicit `self-build-proof: fallback-event=<kind>` markers or `self-build-proof: fallback-event=none`. The proof MUST fail closed when any named live-bootstrap placeholder remains, when the legacy provider is selected for a StageX claim, when host-bwrap or checkout/source-discovery fallback appears, or when forbidden host executables run during the protected stage.

#### Scenario: Release evidence binds selected provider kind

- GIVEN a full self-hosting proof bundle whose prerequisites name a selected provider kind
- WHEN release evidence is created from that proof bundle
- THEN `proof_linkage.selected_provider_kind` equals the proof bundle's selected provider kind
- AND verification fails if either side is missing, unknown, or mismatched
