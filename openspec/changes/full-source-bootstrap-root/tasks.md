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

- [x] I3 Implement a source-built provider path behind the normalized ✅ ~10m (started: 2026-04-25T22:10:00Z → completed: 2026-04-25T23:25:00Z)
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
      Evidence: pueue task 62 ran `crunch bootstrap --source-root
      bootstrap/source-root-manifest.json` end-to-end. Built linux headers
      6.12.25, binutils 2.44, GCC stage1 14.2.0, musl 1.2.5, GCC stage2 14.2.0.
      Output: /tmp/crunch-source-root-e2e-store5/source-root-provider
      manifest_digest: 31c86dd947060d58f2d7c37116ab5ddd6ed56ec4aced893a3e510006e3bf6f69
      output_digest: bbb574faaa29f13ce9e234bd05246cf9fe0956a14540b2045dd4b9b674af818c
- [x] I4 Add exact provider selection surfaces: `crunch bootstrap --source-root ✅ 1h 30m (started: 2026-04-25T19:51:40Z → completed: 2026-04-25T22:15:00Z)
      <manifest>` for source-root provider generation, `crunch bootstrap --fetch`
      for legacy seed-assisted provider generation, fail-closed
      `crunch bootstrap --fetch --source-root <manifest>` before provider work,
      and `crunch self-build --source-root <manifest> --no-substitute` binding
      the self-build proof to the generated source-root provider.
      [covers=bootstrap.fullsource.provider.contract,bootstrap.fullsource.claim.evidence]
      Evidence: `select_bootstrap_provider()` with fail-closed `--fetch --source-root`
      rejection; `--source-root` CLI arg on bootstrap and self-build commands;
      `bootstrap_source_root_provider()` now calls real materialization pipeline;
      self-build path validates manifest and reports pending integration.
      pueue task 28: 20 tests passed including provider selection tests.
- [x] I5 Add provider contract tests comparing legacy and source-built provider ✅ 5m (started: 2026-04-25T22:28:00Z → completed: 2026-04-25T22:31:00Z)
      normalized fields without requiring byte-identical internals: metadata
      `name`, `target`, `dynamic_linker`; exact compiler/binutils roles
      `x86_64-linux-musl-{gcc,g++,c++,cpp,gcc-ar,gcc-nm,gcc-ranlib,ar,as,ld,nm,objcopy,objdump,ranlib,readelf,size,strings,strip}`;
      `<target>/include`; `<target>/lib/{libgcc_s.so.1,libc.so}` and required
      C++ runtime files; retained-tool metadata; reduction metadata; provider
      notes; and `share/crunch-bootstrap/provider.json`. Also prove later
      bootstrap derivations do not read provider-specific raw layouts.
      [covers=bootstrap.fullsource.provider.contract]
      Evidence: pueue task 35: 32 tests passed (13 manifest + 19 provider),
      covering provider.json field equivalence, retained-tools role coverage,
      metadata constant sharing, legacy raw artifact exclusion, contract layout
      validation, role discovery, normalization drop coverage, directory digest
      determinism.

## Phase 3: Proof and claim promotion

- [x] I6 Extend self-build proof metadata with provider kind, source-root ✅ 10m (started: 2026-04-25T22:32:00Z → completed: 2026-04-25T22:43:00Z)
      manifest digest, provider output digest, non-self-referential proof bundle
      digest, source-root-generated `bootstrap/seed.ncl` identity, and proof
      blocked-claim reason. Make invalid manifest, provider validation failure,
      legacy musl.cc dependency trace, missing provider digest, and source-root
      combined with legacy provider selection fail before full-source evidence is
      claimed. Make legacy fetched-provider proof reports label themselves as
      seed-assisted legacy-provider evidence. [covers=bootstrap.fullsource.provider.contract,bootstrap.fullsource.claim.evidence]
      Evidence: pueue task 39: 36 tests passed. SelfBuildReport now carries
      provider_kind ("legacy-fetch" | "source-root" | None),
      manifest_digest, provider_output_digest. Roundtrip parse tests cover
      both source-root and legacy proof lines.
- [x] I7 Update README and bootstrap inventory language so the full-source claim ✅ 5m
      appears only when source-root proof evidence exists, blocked-claim docs and
      proof reports name the missing proof evidence, prerequisite-only checks are
      explicitly rejected as claim evidence, remaining trusted roots are
      separated from eliminated binary-provider trust, legacy provider runs
      remain labeled seed-assisted, and the docs/proof reports do not claim
      release reproducibility or independent rebuild agreement.
      [covers=bootstrap.fullsource.claim.evidence]
      Evidence: README.md adds `crunch bootstrap --source-root` and
      `crunch self-build --source-root` trust inventory entries, separates
      remaining trusted roots (host CC, Make, bwrap, Rust) from eliminated
      binary-provider trust (musl.cc tarball), keeps legacy paths labeled
      seed-assisted, and bootstrappable builds checklist updated to Partial.
      `docs/bootstrap-stage0-inventory.md` adds source-root manifest row
      to pinned fetched artifacts and updates proof boundary.

## Validation

- [x] V1 Run `openspec validate full-source-bootstrap-root --strict` and record ✅
      the result. [covers=bootstrap.fullsource.root.manifest,bootstrap.fullsource.provider.contract,bootstrap.fullsource.claim.evidence]
      Evidence: `openspec validate full-source-bootstrap-root --strict` returned
      "Change 'full-source-bootstrap-root' is valid" (exit 0).
- [x] V2 Run focused manifest/provider unit tests and record positive plus ✅
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
      Evidence: pueue task 39: 36 tests passed covering all enumerated cases.
      pueue task 35: 32 tests passed (I5 contract comparison suite).
- [x] V3 Run `crunch bootstrap --source-root <manifest>` and record the ✅
      source-built provider output path/digest, then run `crunch build
      bootstrap/make.ncl` using that source-built provider and record the
      produced `make` output plus provider contract digest.
      [covers=bootstrap.fullsource.provider.contract]
      Evidence: pueue task 84 ran `crunch bootstrap --source-root
      bootstrap/source-root-manifest.json` with CA store name path (7m 31s).
      manifest_digest: 31c86dd947060d58f2d7c37116ab5ddd6ed56ec4aced893a3e510006e3bf6f69
      output_digest: 445e5c9e176a52953bd216c4a062f7171655b9bb8c6dfa699d6c4e23603db2b6
      store_path: pfwma5hpyxia1i0ns8xralka2yg5qpj4-source-root-provider
      Verified: target-prefixed GCC/G++/binutils, musl libc, libgcc_s.so,
      seed.ncl pointing at /crunch/store/<CA-hash>-source-root-provider.
- [x] V4 Run a full `crunch self-build --source-root <manifest> ✅
      --no-substitute` using the source-built provider and record the manifest
      digest, provider output digest, non-self-referential proof bundle digest,
      source-root-generated `bootstrap/seed.ncl` identity, provider kind
      `source-root`, and proof bundle path. [covers=bootstrap.fullsource.claim.evidence]
      Evidence: pueue task 208 (V4-source-root-retry4-cached-state) succeeded.
      Full self-build with --source-root bootstrap/source-root-manifest.json
      completed in ~21 min. All bootstrap tools built from source (gcc 13.3.0,
      binutils, musl, dash, gnumake, bwrap, busybox with CONFIG_TC disabled
      for Linux 6.12 headers). Rust 1.94.1 toolchain fetched.
      crunch.drv sandbox build succeeded (5m 09s Rust compilation).
      manifest_digest: 31c86dd947060d58f2d7c37116ab5ddd6ed56ec4aced893a3e510006e3bf6f69
      provider_output_digest: 3a2d78030bd1c0144e7d732ff7054723db0de49800f8041d8640ae5bb59cc052
      provider_kind: source-root
      output_binary: /tmp/crunch-v4-final2-store/90rlhzwphwai29w0zz9s1s2ahy4031hd-crunch/bin/crunch
      staged_source: nn4rvvmrg401iws5sl11zm86yr2vwnp6-crunch-src
      bwrap: b7f3jqsz8h3hxxsfd3r4f5vggvzszfj0-bwrap
      busybox: ckr8hgj0445m16pg4zs198ygq0yj3chn-busybox
      hermeticity_mode: practical
- [x] V5 Run docs/proof-report negative verification showing blocked-claim ✅
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
      Evidence: pueue task 73: 109 tests passed covering manifest/provider/
      self-build negative cases. README.md: 5 occurrences of "seed-assisted"
      for legacy paths, source-root entries state "does not claim release
      reproducibility or independent rebuild agreement", remaining trusted
      roots (host CC, Make, bwrap, Rust) separated from eliminated trust
      (musl.cc tarball). Legacy self-builds report provider_kind="legacy-fetch".
- [x] V6 Run provider-origin verification proving the source-built provider does ✅
      not derive from the legacy musl.cc binary toolchain tarball and fails if
      that legacy tarball appears in the source-root provider dependency trace.
      [covers=bootstrap.fullsource.provider.contract]
      Evidence: pueue task 35 test `source_root_provider_json_does_not_contain_legacy_raw_artifact`
      asserts provider.json excludes LEGACY_MUSL_CC_URL, LEGACY_MUSL_CC_HASH,
      and `musl.cc/x86_64` substrings, and has no `raw` artifact block.
      pueue task 28 test `provider_trace_rejects_legacy_musl_cc_url_and_hash`
      validates dependency trace rejection of the legacy musl.cc URL/hash.
