# Tasks: full-source bootstrap root

## Phase 1: Source-root manifest foundation

- [x] I1 Add a pure manifest model and validator for full-source bootstrap ✅ 2m 37s (started: 2026-04-25T19:44:50Z → completed: 2026-04-25T19:47:27Z)
      roots, including version, artifacts, patches, extraction rules,
      network trust roots with rationale, expected outputs with kind and
      required contract role, explicit trust notes, provenance, trust-note
      scope/rationale, non-BLAKE3 reason validation, and digest validation for
      every artifact/patch/output/network/trust-note entry.
      [covers=bootstrap.fullsource.root.manifest]
- [x] I2 Add positive and negative manifest fixtures covering valid manifests, ✅ 2m 37s (started: 2026-04-25T19:44:50Z → completed: 2026-04-25T19:47:27Z)
      missing, invalid, or unsupported manifest versions, missing artifact/patch/
      output/network/trust-note digests, missing provenance fields, missing
      trust-note scope/rationale, missing network trust-root rationale, missing
      or invalid extraction rules, undeclared patches, missing/invalid expected
      output kind or required contract role, unexpected/unmanifested provider
      outputs, unmanifested network trust roots, trust-note digest/provenance
      failures, non-BLAKE3 digest reasons, legacy musl.cc URL/hash rejection,
      and output mismatch diagnostics.
      [covers=bootstrap.fullsource.root.manifest]
      Evidence: pueue task 59 passed 13 positive/negative fixtures in
      `bootstrap_source_root::tests`, including valid manifest acceptance,
      missing/unsupported version rejection, missing digest, non-BLAKE3 without
      reason, missing extraction, undeclared patch, missing network rationale,
      missing trust-note scope, missing output role, legacy musl.cc URL/hash,
      unmanifested URL, and unexpected provider output role.

## Phase 2: Source-built provider

- [ ] I3 Implement a source-built provider path behind the normalized
      `bootstrap/seed.ncl` contract without deriving from the legacy musl.cc
      binary toolchain tarball. The provider MUST consume the design source set
      (Linux UAPI headers, musl source, binutils source, GCC source/runtime
      source) and execute the fixed phases: validate manifest, fetch/read
      artifacts, verify digests, unpack by manifest extraction rule, apply
      declared patches in order, build/install into a staging root, normalize
      into the seed contract, write `share/crunch-bootstrap/provider.json`, and
      validate normalized output. The implementation MUST scan manifest artifact
      URLs, network trust roots, provider metadata, and recorded dependency trace
      for the legacy musl.cc URL/hash and fail before provider acceptance if
      found. [covers=bootstrap.fullsource.provider.contract]
- [ ] I4 Add exact provider selection surfaces: `crunch bootstrap --source-root
      <manifest>` for source-root provider generation, `crunch bootstrap --fetch`
      for legacy seed-assisted provider generation, fail-closed
      `crunch bootstrap --fetch --source-root <manifest>` before provider work,
      and `crunch self-build --source-root <manifest> --no-substitute` binding
      the self-build proof to the generated source-root provider.
      [covers=bootstrap.fullsource.provider.contract,bootstrap.fullsource.claim.evidence]
- [ ] I5 Add provider contract tests comparing legacy and source-built provider
      normalized fields without requiring byte-identical internals: metadata
      `name`, `target`, `dynamic_linker`; exact compiler/binutils roles
      `x86_64-linux-musl-{gcc,g++,c++,cpp,gcc-ar,gcc-nm,gcc-ranlib,ar,as,ld,nm,objcopy,objdump,ranlib,readelf,size,strings,strip}`;
      `<target>/include`; `<target>/lib/{libgcc_s.so.1,libc.so}` and required
      C++ runtime files; retained-tool metadata; reduction metadata; provider
      notes; and `share/crunch-bootstrap/provider.json`. Also prove later
      bootstrap derivations do not read provider-specific raw layouts.
      [covers=bootstrap.fullsource.provider.contract]

## Phase 3: Proof and claim promotion

- [ ] I6 Extend self-build proof metadata with provider kind, source-root
      manifest digest, provider output digest, non-self-referential proof bundle
      digest, source-root-generated `bootstrap/seed.ncl` identity, and proof
      blocked-claim reason. Make invalid manifest, provider validation failure,
      legacy musl.cc dependency trace, missing provider digest, and source-root
      combined with legacy provider selection fail before full-source evidence is
      claimed. Make legacy fetched-provider proof reports label themselves as
      seed-assisted legacy-provider evidence. [covers=bootstrap.fullsource.provider.contract,bootstrap.fullsource.claim.evidence]
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
      unmanifested network trust-root rejection, network trust root missing
      rationale, trust notes missing scope or rationale, missing/invalid expected
      output kind or required contract role, non-BLAKE3 digest without reason,
      ambiguous legacy-plus-source-root provider selection fail-closed behavior,
      standalone `crunch bootstrap --source-root <manifest>` materialization
      output, legacy musl.cc URL/hash
      dependency-trace rejection, and legacy-vs-source comparison of exact
      normalized provider fields: metadata `name`/`target`/`dynamic_linker`,
      exact compiler/binutils roles, `<target>/include`, `<target>/lib` required
      libraries, retained-tool metadata, reduction metadata, provider notes, and
      `share/crunch-bootstrap/provider.json`.
      [covers=bootstrap.fullsource.root.manifest,bootstrap.fullsource.provider.contract,bootstrap.fullsource.claim.evidence]
- [ ] V3 Run `crunch bootstrap --source-root <manifest>` and record the
      source-built provider output path/digest, then run `crunch build
      bootstrap/make.ncl` using that source-built provider and record the
      produced `make` output plus provider contract digest.
      [covers=bootstrap.fullsource.provider.contract]
- [ ] V4 Run a full `crunch self-build --source-root <manifest>
      --no-substitute` using the source-built provider and record the manifest
      digest, provider output digest, non-self-referential proof bundle digest,
      source-root-generated `bootstrap/seed.ncl` identity, provider kind
      `source-root`, and proof bundle path. [covers=bootstrap.fullsource.claim.evidence]
- [ ] V5 Run docs/proof-report negative verification showing blocked-claim
      reports name the missing proof evidence, successful full-source bootstrap
      maturity reports include manifest digest, provider output digest, and proof
      bundle digest, prerequisite-only checks are not accepted as full-source
      bootstrap claim evidence, invalid manifest/provider
      validation/legacy musl.cc trace/missing provider digest fail before
      full-source evidence is claimed, legacy fetched-provider proof reports
      remain labeled seed-assisted, docs separate remaining trusted roots from
      eliminated binary-provider trust, and no full-source-root artifact claims
      release reproducibility or independent rebuild agreement.
      [covers=bootstrap.fullsource.provider.contract,bootstrap.fullsource.claim.evidence]
- [ ] V6 Run provider-origin verification proving the source-built provider does
      not derive from the legacy musl.cc binary toolchain tarball and fails if
      that legacy tarball appears in the source-root provider dependency trace.
      [covers=bootstrap.fullsource.provider.contract]
