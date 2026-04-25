## ADDED Requirements

### Requirement: Release evidence may carry independent agreement evidence

Crunch MUST allow release evidence or verification directories to carry an independent rebuild agreement report without making that report mandatory for bundle-local integrity verification.
ID: release.evidence.independent.agreement.attachment

Bundle verification MUST continue to distinguish basic bundle integrity, self-proof validity, matching external witnesses, and independent rebuild agreement. Missing agreement evidence MUST NOT invalidate a basic release-evidence bundle, but any present agreement report MUST verify against the release attestation and witness material it names. A release-evidence bundle MAY store the report at `independent-agreement/agreement-report.json`; a verification directory MAY store it at `agreement-report.json`. Discovery MUST reject any additional agreement-report filename for the same release to avoid ambiguous attachments.

#### Scenario: Bundle without agreement remains basic-valid

- GIVEN a release evidence bundle with valid source, binary, manifest, and proof
  artifacts
- AND no independent agreement report
- WHEN bundle verification runs
- THEN basic bundle verification succeeds
- AND the output says independent agreement evidence is absent

#### Scenario: Mismatched agreement report is rejected

- GIVEN a verification directory with an agreement report naming a different
  release-attestation digest
- WHEN release verification loads the report
- THEN verification fails for the agreement attachment
- AND the diagnostic names the digest mismatch
