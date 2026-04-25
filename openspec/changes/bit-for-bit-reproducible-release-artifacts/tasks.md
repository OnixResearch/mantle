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
- [x] I2 Add report canonicalization and stable report identity digest tests plus negative fixtures for one-byte ✅ 0m 56s (started: 2026-04-25T19:03:57Z → completed: 2026-04-25T19:04:53Z)
      drift, missing artifact, output-name drift, and proof-linkage mismatch.
      The digest test must assert identical semantic reports produce the same
      BLAKE3 digest over canonical compact JSON bytes.
      [covers=release.evidence.reproducible.report]
      Evidence: `cargo test -p crunch-release-core reproducibility_report` passed
      in pueue task 51 (10 filtered tests), including stable canonical-byte and
      BLAKE3 report identity assertions, and `cargo test -p
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

- [x] I5 Extend `release verify` to load optional reproducibility reports, ✅ 1m 28s (started: 2026-04-25T19:16:19Z → completed: 2026-04-25T19:17:47Z)
      validate canonical report encoding and report digest, verify report linkage
      to the bundle's release identifier, source archive digest, and proof bundle
      digest, and require the report artifact set to match the published release
      artifact set before any reproducible-release claim is emitted.
      [covers=release.evidence.reproducible.report,release.evidence.reproducible.claim.gate,release.verification.tech.reproducibility.status]
      Evidence: `cargo test -p crunch --test release_cli release_verify_`
      passed in pueue task 21 (19 tests), covering matched report loading,
      non-canonical report rejection, linkage mismatch, artifact-set
      mismatch, and existing bundle/linkage drift failures.
- [x] I6 Extend `release verify` JSON output with reproducibility status ✅ 0m 55s (started: 2026-04-25T19:18:27Z → completed: 2026-04-25T19:19:22Z)
      `absent`, `matched`, or `mismatched`, and add an option for callers to
      require matched reproducibility evidence.
      [covers=release.verification.tech.reproducibility.status,release.evidence.reproducible.claim.gate]
      Evidence: `cargo test -p crunch --test release_cli release_verify_`
      passed in pueue task 23 (24 tests), covering JSON statuses `absent`,
      `matched`, `mismatched`, and `--require-reproducible` success/failure.
- [x] I7 Implement release evidence bundle creation/package placement for ✅ 6m 50s (started: 2026-04-25T19:20:07Z → completed: 2026-04-25T19:26:57Z)
      reproducibility evidence: `crunch release create` MUST copy an optional
      canonical reproducibility report sidecar into the bundle, record its
      bundle-local path and BLAKE3 digest in manifest/report metadata, and keep
      bundles without that sidecar verifying as ordinary non-reproducible release
      evidence. [covers=release.evidence.reproducible.report,release.evidence.reproducible.claim.gate]
      Evidence: `cargo test -p crunch --test release_cli release_create`
      passed in pueue task 30 (4 tests), `cargo test -p crunch --test
      release_cli release_verify_` passed in pueue task 31 (24 tests),
      `cargo test -p crunch-release-core` passed in pueue task 34
      (16 tests), and `cargo check -p crunch-release-core --target
      wasm32-unknown-unknown` passed in pueue task 33.
- [x] I8 Update release manifests/report summaries so ordinary bundle-local ✅ 1m 24s (started: 2026-04-25T19:27:35Z → completed: 2026-04-25T19:28:59Z)
      integrity remains separate from reproducible-release evidence and the
      manifest cannot imply reproducibility without a verified report.
      [covers=release.evidence.reproducible.claim.gate]
      Evidence: `cargo test -p crunch --test release_cli release_` passed in
      pueue task 36 (41 tests), including assertions that `claim_scope`
      remains `packaged-integrity-evidence`, no `reproducible_release`
      manifest flag is emitted, and reproducibility status/report data stays
      separate in release verify output.
- [x] I9 Update release docs to reserve the bit-for-bit reproducible release ✅ 1m 13s (started: 2026-04-25T19:30:09Z → completed: 2026-04-25T19:31:22Z)
      label for bundles with a verified reproducibility report whose artifact
      set matches the published release artifact set.
      [covers=release.evidence.reproducible.claim.gate]
      Evidence: docs audit passed in pueue task 38, checking README,
      docs/operator-workflows.md, and docs/bootstrap-stage0-inventory.md
      for `--require-reproducible`, explicit bit-for-bit label gating,
      separate reproducibility status, and no bundle-local implication.

## Validation

- [x] V1 Run `openspec validate bit-for-bit-reproducible-release-artifacts --strict` ✅ 0m 7s (started: 2026-04-25T19:31:41Z → completed: 2026-04-25T19:31:48Z)
      and record the result. [covers=release.evidence.reproducible.report,release.evidence.reproducible.cli,release.evidence.reproducible.claim.gate,release.verification.tech.reproducibility.status]
      Evidence: `openspec validate bit-for-bit-reproducible-release-artifacts --strict` returned `Change 'bit-for-bit-reproducible-release-artifacts' is valid`.
- [x] V2 Run `cargo test -p crunch-release-core reproducibility_report` proving ✅ 0m 0s (started: 2026-04-25T19:33:39Z → completed: 2026-04-25T19:33:39Z)
      canonical serialization, byte-identical success, and one-byte drift
      failure. [covers=release.evidence.reproducible.report]
      Evidence: pueue task 40 passed: 10 filtered reproducibility report tests covered canonical serialization, stable BLAKE3 report identity over canonical compact JSON, matched success, one-byte/digest drift, missing rebuilt artifact, output-name drift, and proof-linkage mismatch.
- [x] V3 Run `cargo test -p crunch --test release_cli release_` for ✅ 0m 0s (started: 2026-04-25T19:33:39Z → completed: 2026-04-25T19:33:39Z)
      matched, absent-required, byte-length drift, digest drift,
      missing-artifact, output-name drift, source archive drift, manifest drift,
      packaging-metadata drift, non-canonical report, proof-linkage mismatch,
      prerequisite-only proof input, bundle-linkage mismatch, and artifact-set
      mismatch outcomes. [covers=release.evidence.reproducible.cli,release.evidence.reproducible.report,release.evidence.reproducible.claim.gate,release.verification.tech.reproducibility.status]
      Evidence: pueue task 36 passed `cargo test -p crunch --test release_cli release_` (41 tests) across matched, absent-required, byte-length drift, digest drift, missing artifact, output-name drift, source archive drift, manifest drift, packaging metadata separation, non-canonical report, proof-linkage mismatch, prerequisite-only proof input, bundle-linkage mismatch, and artifact-set mismatch paths; focused `release_reproduce` subset also passed in pueue task 42 (5 tests).
- [x] V4 Run `cargo test -p crunch --test release_cli release_verify_json` proving ✅ 0m 0s (started: 2026-04-25T19:33:39Z → completed: 2026-04-25T19:33:39Z)
      reproducibility status remains separate from signature trust, witness
      agreement, and basic bundle integrity, including a case where
      signatures/witnesses remain valid while reproducibility status is
      `mismatched`. [covers=release.verification.tech.reproducibility.status]
      Evidence: pueue task 43 passed 3 JSON tests for reproducibility statuses `absent`, `matched`, and `mismatched`, with status reported separately from the rest of the release-verify envelope.
- [x] V5 Run docs/manifest reproducibility claim audit ✅ 0m 0s (started: 2026-04-25T19:33:39Z → completed: 2026-04-25T19:33:39Z)
      proving ordinary bundle-local integrity stays separate from
      reproducible-release evidence, and absent or unverified reports cannot
      imply bit-for-bit reproducibility. [covers=release.evidence.reproducible.claim.gate]
      Evidence: pueue task 44 passed a deterministic docs/manifest audit over README.md, docs/operator-workflows.md, and docs/bootstrap-stage0-inventory.md for `--require-reproducible`, explicit label gating, ordinary-evidence narrowing, separate status text, and no bundle-local implication.
