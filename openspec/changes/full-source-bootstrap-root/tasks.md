# Tasks: full-source bootstrap root

## Phase 1: Source-root manifest foundation

- [ ] I1 Add a pure manifest model and validator for full-source bootstrap
      roots, including version, artifact digests, patches, extraction rules,
      expected outputs, and explicit trust notes.
      [covers=bootstrap.fullsource.root.manifest]
- [ ] I2 Add positive and negative manifest fixtures covering valid manifests,
      missing, invalid, or unsupported manifest versions, missing artifact/patch/
      output digests, missing provenance fields, missing or invalid extraction
      rules, undeclared patches, unexpected/unmanifested provider outputs,
      unmanifested network trust roots, trust-note digest/provenance failures,
      non-BLAKE3 digest reasons, and output mismatch diagnostics.
      [covers=bootstrap.fullsource.root.manifest]

## Phase 2: Source-built provider

- [ ] I3 Implement a source-built provider path behind the normalized
      `bootstrap/seed.ncl` contract without deriving from the legacy musl.cc
      binary toolchain tarball. [covers=bootstrap.fullsource.provider.contract]
- [ ] I4 Add the provider selection surface (`crunch bootstrap --source-root` or
      equivalent config/CLI entry point) and fail closed when both legacy and
      source-root providers are requested ambiguously.
      [covers=bootstrap.fullsource.provider.contract,bootstrap.fullsource.claim.evidence]
- [ ] I5 Add provider contract tests comparing legacy and source-built provider
      normalized fields without requiring byte-identical internals: target-
      prefixed tool paths, headers, libraries, retained-tool metadata, reduction
      metadata, and provider notes. Also prove later bootstrap derivations do
      not read provider-specific raw layouts.
      [covers=bootstrap.fullsource.provider.contract]

## Phase 3: Proof and claim promotion

- [ ] I6 Extend self-build proof metadata with provider kind, source-root
      manifest digest, provider output digest, and proof bundle digest, and make
      legacy fetched-provider proof reports label themselves as seed-assisted
      legacy-provider evidence. [covers=bootstrap.fullsource.provider.contract,bootstrap.fullsource.claim.evidence]
- [ ] I7 Update README and bootstrap inventory language so the full-source claim
      appears only when source-root proof evidence exists, blocked-claim docs and
      proof reports name the missing proof evidence, prerequisite-only checks are
      explicitly rejected as claim evidence, remaining trusted roots are
      separated from eliminated binary-provider trust, legacy provider runs
      remain labeled seed-assisted, and the docs/proof reports do not claim
      release reproducibility or independent rebuild agreement.
      [covers=bootstrap.fullsource.claim.evidence]

## Validation

- [ ] V1 Run `openspec validate full-source-bootstrap-root --strict` and record
      the result. [covers=bootstrap.fullsource.root.manifest,bootstrap.fullsource.provider.contract,bootstrap.fullsource.claim.evidence]
- [ ] V2 Run focused manifest/provider unit tests and record positive plus
      negative case output, including missing/invalid/unsupported manifest
      version rejection, unexpected/unmanifested provider output rejection,
      unmanifested network trust-root rejection, ambiguous legacy-plus-source-
      root provider selection fail-closed behavior, and legacy-vs-source
      comparison of exact normalized provider fields: target-prefixed tool
      paths, headers, libraries, retained-tool metadata, reduction metadata, and
      provider notes.
      [covers=bootstrap.fullsource.root.manifest,bootstrap.fullsource.provider.contract,bootstrap.fullsource.claim.evidence]
- [ ] V3 Run `crunch build bootstrap/make.ncl` using the source-built provider
      and record the produced `make` output plus provider contract digest.
      [covers=bootstrap.fullsource.provider.contract]
- [ ] V4 Run a full `crunch self-build --no-substitute` using the source-built
      provider and record the manifest digest, provider output digest, proof
      bundle digest, and proof bundle path. [covers=bootstrap.fullsource.claim.evidence]
- [ ] V5 Run docs/proof-report negative verification showing blocked-claim
      reports name the missing proof evidence, prerequisite-only checks are not
      accepted as full-source bootstrap claim evidence, legacy fetched-provider
      proof reports remain labeled seed-assisted, docs separate remaining trusted
      roots from eliminated binary-provider trust, and no full-source-root
      artifact claims release reproducibility or independent rebuild agreement.
      [covers=bootstrap.fullsource.provider.contract,bootstrap.fullsource.claim.evidence]
- [ ] V6 Run provider-origin verification proving the source-built provider does
      not derive from the legacy musl.cc binary toolchain tarball and fails if
      that legacy tarball appears in the source-root provider dependency trace.
      [covers=bootstrap.fullsource.provider.contract]
