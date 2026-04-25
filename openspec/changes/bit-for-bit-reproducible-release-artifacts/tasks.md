# Tasks: bit-for-bit reproducible release artifacts

## Phase 1: Reproducibility report core

- [ ] I1 Add pure reproducibility-report types with canonical compact JSON over
      release identifier, source digest, proof digest, rebuild command identity,
      artifact names, byte lengths, BLAKE3 digests, and comparison results.
      [covers=release.evidence.reproducible.report]
- [ ] I2 Add report canonicalization tests plus negative fixtures for one-byte
      drift, missing artifact, output-name drift, and proof-linkage mismatch.
      [covers=release.evidence.reproducible.report]

## Phase 2: Rebuild and compare workflow

- [ ] I3 Add a release reproducibility CLI workflow that rebuilds published
      artifacts into an isolated output area and compares only the manifest's
      named release artifacts. [covers=release.evidence.reproducible.cli]
- [ ] I4 Add fail-closed diagnostics for missing rebuilt artifact, output-name
      drift, byte-length drift, digest drift, source archive drift, manifest
      drift, non-canonical report encoding, proof-linkage mismatch, packaging
      metadata drift, and prerequisite-only proof input.
      [covers=release.evidence.reproducible.cli]

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
- [ ] I7 Update release manifests/report summaries so ordinary bundle-local
      integrity remains separate from reproducible-release evidence and the
      manifest cannot imply reproducibility without a verified report.
      [covers=release.evidence.reproducible.claim.gate]
- [ ] I8 Update release docs to reserve the bit-for-bit reproducible release
      label for bundles with a verified reproducibility report whose artifact
      set matches the published release artifact set.
      [covers=release.evidence.reproducible.claim.gate]

## Validation

- [ ] V1 Run `openspec validate bit-for-bit-reproducible-release-artifacts
      --strict` and record the result. [covers=release.evidence.reproducible.report,release.evidence.reproducible.cli,release.evidence.reproducible.claim.gate,release.verification.tech.reproducibility.status]
- [ ] V2 Run reproducibility report core tests proving canonical serialization,
      byte-identical success, and one-byte drift failure.
      [covers=release.evidence.reproducible.report]
- [ ] V3 Run release CLI reproducibility tests for matched, absent-required,
      byte-length drift, digest drift, missing-artifact, output-name drift,
      source archive drift, manifest drift, packaging-metadata drift,
      non-canonical report, proof-linkage mismatch, prerequisite-only proof
      input, bundle-linkage mismatch, and artifact-set mismatch outcomes.
      [covers=release.evidence.reproducible.cli,release.evidence.reproducible.report,release.evidence.reproducible.claim.gate,release.verification.tech.reproducibility.status]
- [ ] V4 Run release verification JSON tests proving reproducibility status
      remains separate from signature trust, witness agreement, and basic bundle
      integrity, including a case where signatures/witnesses remain valid while
      reproducibility status is `mismatched`.
      [covers=release.verification.tech.reproducibility.status]
- [ ] V5 Run docs and manifest/report-summary audits proving ordinary
      bundle-local integrity stays separate from reproducible-release evidence,
      and absent or unverified reports cannot imply bit-for-bit reproducibility.
      [covers=release.evidence.reproducible.claim.gate]
