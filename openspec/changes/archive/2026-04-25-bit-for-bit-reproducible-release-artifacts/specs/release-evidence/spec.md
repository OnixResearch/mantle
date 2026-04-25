## ADDED Requirements

### Requirement: Canonical reproducible release report

Crunch MUST define a canonical reproducible release report that records byte-for-byte comparison evidence for every published release artifact.
ID: release.evidence.reproducible.report

The report MUST bind the release identifier, source archive digest, proof bundle digest, rebuild command identity, artifact names, byte lengths, BLAKE3 digests, and comparison result. The report identity MUST be the BLAKE3 digest of canonical compact JSON bytes.

#### Scenario: Byte-identical report is stable

- GIVEN two reproducibility checks over the same published and rebuilt artifact
  bytes
- WHEN the report is serialized
- THEN the canonical bytes are identical
- AND the report digest is identical

#### Scenario: One-byte drift is recorded

- GIVEN a rebuilt binary differs from the published binary by one byte
- WHEN reproducibility comparison runs
- THEN the report marks that artifact as mismatched
- AND it records the expected and observed BLAKE3 digests

### Requirement: CLI rebuilds and compares release artifacts

Crunch MUST provide a release reproducibility workflow that rebuilds published artifacts into a separate output area and compares the rebuilt bytes against the bundled release artifacts.
ID: release.evidence.reproducible.cli

The workflow MUST fail closed on missing artifacts, output-name drift, byte-length drift, digest drift, non-canonical report encoding, or proof-linkage mismatch. A successful prerequisite check MUST NOT count as a reproducible release check.

#### Scenario: Byte-identical rebuild succeeds

- GIVEN a release evidence bundle with published binary artifacts
- AND a reproducibility workflow rebuilds the same output names with identical
  bytes
- WHEN the release reproducibility command runs
- THEN it exits successfully
- AND it writes a canonical reproducible release report

#### Scenario: Missing rebuilt artifact fails

- GIVEN the published release bundle names `crunch-x86_64-linux`
- AND the rebuild output directory does not contain that artifact
- WHEN reproducibility comparison runs
- THEN the command exits non-zero
- AND the diagnostic names the missing rebuilt artifact

### Requirement: Reproducible release claim is evidence-gated

Crunch MUST reserve the bit-for-bit reproducible release claim for release evidence that includes a verified reproducibility report whose artifact set matches the published release artifact set.
ID: release.evidence.reproducible.claim.gate

Docs, manifests, and verifier output MUST keep ordinary bundle-local integrity separate from reproducible-release evidence. A release MUST NOT be labeled bit-for-bit reproducible when only self-proof or prerequisite evidence is present.

#### Scenario: Ordinary bundle is not labeled reproducible

- GIVEN a release evidence bundle verifies basic integrity and self-proof
- BUT it has no reproducibility report
- WHEN release verification reports classes
- THEN it does not label the release bit-for-bit reproducible
- AND it names the missing reproducibility evidence

#### Scenario: Verified report enables reproducible label

- GIVEN a release evidence bundle with a verified reproducibility report
- AND the report artifact set matches the published artifact set
- WHEN release verification reports classes
- THEN it may label the release bit-for-bit reproducible
- AND it includes the reproducibility report digest
