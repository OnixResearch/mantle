# Tasks: bit-for-bit reproducible release artifacts

## Phase 1: Reproducibility report core

- [x] I1 Add pure reproducibility-report types with canonical compact JSON over ✅ 6m 28s (started: 2026-04-25T18:53:32Z → completed: 2026-04-25T19:00:00Z)
      release identifier, source digest, proof digest, rebuild command identity,
      artifact names, byte lengths, BLAKE3 digests, comparison results, and a
      report identity that the core computes and exposes as the BLAKE3 digest of
      the canonical compact JSON bytes. [covers=release.evidence.reproducible.report]
      Evidence: baseline `cargo test -p crunch-release-core` passed in pueue task 37
      after adding build-env PATH; post-change `cargo test -p crunch-release-core`
      passed in pueue task 45 (11 passed), `cargo check -p
      crunch-release-core --target wasm32-unknown-unknown` passed in pueue task 46,
      and filtered `cargo test -p crunch-release-core reproducibility_report`
      passed in pueue task 49 (5 passed).
- [x] I2 Add report canonicalization tests plus negative fixtures for one-byte ✅ 0m 56s (started: 2026-04-25T19:03:57Z → completed: 2026-04-25T19:04:53Z)
      drift, missing artifact, output-name drift, and proof-linkage mismatch.
      [covers=release.evidence.reproducible.report]
      Evidence: `cargo test -p crunch-release-core reproducibility_report` passed
      in pueue task 51 (10 filtered tests), and `cargo test -p
      crunch-release-core && cargo check -p crunch-release-core --target
      wasm32-unknown-unknown` passed in pueue task 52 (16 unit tests).

## Phase 2: Rebuild and compare workflow

- [x] I3 Add a release reproducibility CLI workflow that rebuilds published ✅ 5m 5s (started: 2026-04-25T19:05:10Z → completed: 2026-04-25T19:10:15Z)
      artifacts into an isolated output area and compares only the manifest's
      named release artifacts. [covers=release.evidence.reproducible.cli]
      Evidence: `cargo test -p crunch --bin crunch release_reproducibility::`
      plus `cargo test -p crunch --test release_cli
      release_reproduce_writes_matched_report_from_isolated_rebuild_output`
      passed in pueue task 55; the focused CLI test passed again after the
      Unix guard in pueue task 56.
- [x] I4 Add fail-closed diagnostics for missing rebuilt artifact, output-name ✅ 4m 59s (started: 2026-04-25T19:10:53Z → completed: 2026-04-25T19:15:52Z)
      drift, byte-length drift, digest drift, source archive drift, manifest
      drift, non-canonical report encoding, proof-linkage mismatch, packaging
      metadata drift, and prerequisite-only proof input.
      [covers=release.evidence.reproducible.cli]
      Evidence: `cargo test -p crunch --test release_cli release_reproduce`
      passed in pueue task 16 (5 tests: matched, missing, byte-length,
      digest, output-name drift), `cargo test -p crunch --bin crunch
      release_reproducibility::` passed in pueue task 13, and
      `cargo test -p crunch --test release_cli release_verify_` passed in
      pueue task 18 (15 bundle/linkage/signature drift tests).

## Phase 3: Verification and docs

- [ ] I5 Extend `release verify` to load optional reproducibility reports,
      validate canonical report encoding and report digest, verify report linkage
      to the bundle's release identifier, source archive digest, and proof bundle
      digest, and require the report artifact set to match the published release
      artifact set before any reproducible-release claim is emitted.
      [covers=release.evidence.reproducible.report,release.evidence.reproducible.claim.gate,release.verification.tech.reproducibility.status]
- [ ] I6 Extend `release verify` JSON output with reproducibility status
      `absent`, `matched`, or `mismatched`, and add an option for callers to
      require matched reproducibility evidence.
      [covers=release.verification.tech.reproducibility.status,release.evidence.reproducible.claim.gate]
- [ ] I7 Implement release evidence bundle creation/package placement for
      reproducibility evidence: `crunch release create` MUST copy an optional
      canonical reproducibility report sidecar into the bundle, record its
      bundle-local path and BLAKE3 digest in manifest/report metadata, and keep
      bundles without that sidecar verifying as ordinary non-reproducible release
      evidence. [covers=release.evidence.reproducible.report,release.evidence.reproducible.claim.gate]
- [ ] I8 Update release manifests/report summaries so ordinary bundle-local
      integrity remains separate from reproducible-release evidence and the
      manifest cannot imply reproducibility without a verified report.
      [covers=release.evidence.reproducible.claim.gate]
- [ ] I9 Update release docs to reserve the bit-for-bit reproducible release
      label for bundles with a verified reproducibility report whose artifact
      set matches the published release artifact set.
      [covers=release.evidence.reproducible.claim.gate]

## Validation

- [ ] V1 Run `openspec validate bit-for-bit-reproducible-release-artifacts
      --strict` and record the result. [covers=release.evidence.reproducible.report,release.evidence.reproducible.cli,release.evidence.reproducible.claim.gate,release.verification.tech.reproducibility.status]
- [ ] V2 Run `cargo test -p crunch-release-core reproducibility_report` proving
      canonical serialization, byte-identical success, and one-byte drift
      failure. [covers=release.evidence.reproducible.report]
- [ ] V3 Run `cargo test -p crunch --test release_cli reproducibility` for
      matched, absent-required, byte-length drift, digest drift,
      missing-artifact, output-name drift, source archive drift, manifest drift,
      packaging-metadata drift, non-canonical report, proof-linkage mismatch,
      prerequisite-only proof input, bundle-linkage mismatch, and artifact-set
      mismatch outcomes. [covers=release.evidence.reproducible.cli,release.evidence.reproducible.report,release.evidence.reproducible.claim.gate,release.verification.tech.reproducibility.status]
- [ ] V4 Run `cargo test -p crunch --test release_cli release_verify_json` proving
      reproducibility status remains separate from signature trust, witness
      agreement, and basic bundle integrity, including a case where
      signatures/witnesses remain valid while reproducibility status is
      `mismatched`. [covers=release.verification.tech.reproducibility.status]
- [ ] V5 Run `cargo test -p crunch --test release_cli reproducibility_docs_audit`
      proving ordinary bundle-local integrity stays separate from
      reproducible-release evidence, and absent or unverified reports cannot
      imply bit-for-bit reproducibility. [covers=release.evidence.reproducible.claim.gate]
