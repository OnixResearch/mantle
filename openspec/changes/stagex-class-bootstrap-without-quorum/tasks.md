# Tasks: StageX-class bootstrap without quorum

## Phase 1: Target contracts

- [ ] I1 Define the StageX-class lineage manifest/model as a pure core contract,
      including seed bytes, closed seed class allowlist with initial value
      `hex0-seed`, instruction set or bytecode language, entry point, I/O
      contract, allowed host-interface surface, checked-in human-readable source,
      source-to-byte reproduction transcript, manual audit note, exact default
      `audit_seed_max_bytes = 4096` stored next to the seed digest in the
      manifest, source artifacts, generated artifacts, transition tools, patches,
      lowercase BLAKE3 hex canonicality for Crunch-owned digests, provenance,
      expected normalized provider outputs, and separate environment assumptions.
      [covers=bootstrap.stagex.lineage.root]
- [ ] I2 Add positive and negative lineage validation fixtures covering accepted
      audited seed lineage, missing seed audit-bound fields, unsupported seed
      class rejection, exact 4096-byte default seed-budget placement, oversized
      seed rejection, missing digests/provenance, uppercase or malformed BLAKE3
      hex rejection, non-BLAKE3 digest accepted only with a valid
      `interoperability_reason` naming the upstream format/protocol,
      non-BLAKE3 digest rejection without `interoperability_reason`, undeclared
      generated artifacts, provider outputs not reachable from the declared
      lineage, forbidden host `cc`/`c++`/`make`/archive tools, Nix store roots,
      legacy musl.cc provider input, and environment assumptions kept outside
      the source lineage. [covers=bootstrap.stagex.lineage.root]
- [ ] I3 Define the no-quorum StageX-class release profile data model and output
      vocabulary so full-source/reproducible evidence can be satisfied while the
      canonical profile result binds proof bundle digest and reproducibility
      report digest, and quorum remains `not_evaluated` or absent.
      [covers=release.evidence.stagex.profile.noquorum]

## Phase 2: Stage0 lineage provider

- [ ] I4 Implement the StageX-class lineage provider path that materializes the
      normalized `bootstrap/seed.ncl` provider from declared seed/source lineage
      inputs without trusting host compiler/build tools, Nix, or the legacy
      fetched provider. [covers=bootstrap.stagex.lineage.provider]
- [ ] I5 Add provider-boundary tests proving the lineage provider exposes the
      required normalized tool/header/library/metadata roles, `crunch build
      bootstrap/make.ncl` succeeds through `bootstrap/seed.ncl` using that
      provider, and later bootstrap derivations do not read stage0-posix,
      live-bootstrap, OCI, temp, or provider-specific raw paths.
      [covers=bootstrap.stagex.lineage.provider]
- [ ] I6 Wire provider selection into bootstrap/self-build without weakening
      legacy paths: legacy fetched provider remains seed-assisted, source-root
      provider remains intermediate evidence, and StageX-class lineage provider
      is the only provider kind accepted by the target profile.
      [covers=bootstrap.stagex.lineage.provider,release.evidence.stagex.profile.rejects.weaker]

## Phase 3: Proof binding

- [ ] I7 Extend self-build proof metadata with `seed_class`,
      `audit_seed_max_bytes`, audited seed digest, lineage manifest digest,
      stage graph digest, normalized provider digest, staged source digest,
      stage1/stage2 crunch binary digests, bootstrap-tool digests,
      protected-exec audit digest, provider kind `stagex-lineage`, proof bundle
      digest, and lowercase BLAKE3 hex canonicality for all Crunch-owned proof
      digests. [covers=bootstrap.stagex.selfbuild.proof]
- [ ] I8 Make StageX-class proof emission fail closed when lineage evidence is
      missing, legacy fetched, host-tool-trusted, unvalidated, prerequisite-only,
      or when protected execution observes undeclared host compiler, build tool,
      archive tool, shell, Nix command, or legacy provider executable execution.
      [covers=bootstrap.stagex.selfbuild.proof,release.evidence.stagex.profile.rejects.weaker]

## Phase 4: Release verification profile

- [ ] I9 Implement `crunch release verify <bundle-dir>
      --require-stagex-no-quorum` and JSON `stagex_no_quorum` output with fields
      `status`, `class`, `quorum_status`, `provider_kind`, `proof_bundle_digest`,
      `reproducibility_report_digest`, `release_id`, `artifact_set_digest`, and
      `failure_reasons`; require verified bundle integrity, StageX-class lineage
      proof, complete byte-identical reproducibility report, exact artifact-set
      match, lowercase BLAKE3 hex canonicality for proof/report/artifact-set
      digests, canonical binding of proof bundle digest plus reproducibility
      report digest, and no quorum-satisfied success condition.
      [covers=release.evidence.stagex.profile.noquorum]
- [ ] I10 Add negative release-profile tests for missing reproducibility report,
      legacy fetched-provider proof, host-tool-trusted source-root proof,
      self-proof-only evidence, prerequisite-only proof, and external witness
      agreement without StageX-class lineage proof. [covers=release.evidence.stagex.profile.rejects.weaker]

## Phase 5: Documentation and evidence

- [ ] I11 Update README and bootstrap inventory docs to define the StageX-class
      no-quorum target, name remaining environmental assumptions, distinguish it
      from seed-assisted/source-root/self-proof evidence, and state that quorum
      policy is intentionally deferred. [covers=bootstrap.stagex.lineage.root,release.evidence.stagex.profile.noquorum]
- [ ] V1 Run `openspec validate stagex-class-bootstrap-without-quorum --strict`
      and record the output. [covers=bootstrap.stagex.lineage.root,bootstrap.stagex.lineage.provider,bootstrap.stagex.selfbuild.proof,release.evidence.stagex.profile.noquorum,release.evidence.stagex.profile.rejects.weaker]
- [ ] V2 Run focused lineage/provider tests and record positive plus negative
      output for accepted audited seed lineage, missing digest/provenance,
      unsupported seed class, exact `audit_seed_max_bytes = 4096` default stored
      beside the seed digest, oversized seed, uppercase/malformed BLAKE3 hex,
      non-BLAKE3 digest accepted with an interoperability reason naming the
      upstream format/protocol, non-BLAKE3 digest rejection without
      interoperability reason, provider outputs not reachable from declared
      lineage, forbidden host compiler/build/archive tools, Nix roots, legacy
      musl.cc rejection, environment-assumption classification, normalized
      provider roles, `crunch build bootstrap/make.ncl` with the lineage
      provider, and raw layout coupling rejection.
      [covers=bootstrap.stagex.lineage.root,bootstrap.stagex.lineage.provider]
- [ ] V3 Run protected-exec proof negative tests showing undeclared host compiler,
      build tool, archive tool, shell, Nix command, and legacy provider execution
      all fail before StageX-class evidence is emitted. [covers=bootstrap.stagex.selfbuild.proof]
- [ ] V4 Run a full self-build proof with the StageX-class lineage provider and
      record `seed_class`, `audit_seed_max_bytes`, audited seed digest, lineage
      manifest digest, stage graph digest, normalized provider digest, staged
      source digest, stage1/stage2 digests, bootstrap-tool digests,
      protected-exec audit digest, provider kind, proof bundle digest, and
      lowercase BLAKE3 hex formatting for all Crunch-owned proof digests.
      [covers=bootstrap.stagex.selfbuild.proof]
- [ ] V5 Run `crunch release verify <bundle-dir> --require-stagex-no-quorum
      --json` and release-profile negative fixtures showing
      `stagex_no_quorum.status`, `class`, `quorum_status`, `provider_kind`,
      `proof_bundle_digest`, `reproducibility_report_digest`, `release_id`,
      `artifact_set_digest`, and `failure_reasons` are correct; digest fields are
      lowercase BLAKE3 hex; `quorum-satisfied` is not emitted for this profile;
      the no-quorum profile is satisfied only with StageX-class lineage proof
      plus a complete verified reproducibility report; and it remains unsatisfied
      for legacy, host-tool, self-proof-only, prerequisite-only, missing-report,
      and witness-only cases. [covers=release.evidence.stagex.profile.noquorum,release.evidence.stagex.profile.rejects.weaker]
- [ ] V6 Run documentation verification showing README and bootstrap inventory
      state quorum remains unsolved/deferred for this profile, do not use
      `quorum-satisfied` as the StageX-class no-quorum success label, do not
      require multi-signer success conditions, and keep seed-assisted/source-root/
      self-proof evidence distinct from StageX-class lineage evidence.
      [covers=bootstrap.stagex.lineage.root,release.evidence.stagex.profile.noquorum,release.evidence.stagex.profile.rejects.weaker]
