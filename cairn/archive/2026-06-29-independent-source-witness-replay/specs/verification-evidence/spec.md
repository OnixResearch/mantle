# Verification Evidence Specification Delta

## ADDED Requirements

### Requirement: Independent source witness replay [r[verification_evidence.independent_source_witness_replay]]

Mantle release witness rebuilds that are requested to support an independent-source claim MUST acquire source bytes from a release-declared external source origin, verify those bytes against the release manifest's BLAKE3 source archive digest before extraction, and fail closed before launching or signing when the origin is absent, unsupported, unreachable, or digest-mismatched.

#### Scenario: release evidence records external source origin

GIVEN an operator creates release evidence with an external source archive URL
WHEN Mantle writes `manifest.json`
THEN the manifest MUST record a source acquisition entry containing the URL and BLAKE3 digest
AND that digest MUST match `source_archive.digest_blake3`.

#### Scenario: independent-source witness fetches before rebuilding

GIVEN a witness request contains a release manifest with an external source acquisition entry
WHEN `mantle release witness-rebuild --require-independent-source` prepares scratch
THEN it MUST fetch the source archive from the recorded external URL
AND it MUST verify the fetched archive's BLAKE3 digest equals the manifest's source archive digest before extracting or launching the workflow.

#### Scenario: missing origin fails closed

GIVEN a witness request contains only the copied bundled source archive and no source acquisition entry
WHEN `mantle release witness-rebuild --require-independent-source` prepares scratch
THEN it MUST exit non-zero before launching the workflow
AND audit metadata or diagnostics MUST state that independent source acquisition metadata is missing.

#### Scenario: digest mismatch fails closed

GIVEN a witness request records an external source archive URL whose fetched bytes differ from `source_archive.digest_blake3`
WHEN `mantle release witness-rebuild --require-independent-source` prepares scratch
THEN it MUST exit non-zero before extracting source or launching the workflow
AND the diagnostic MUST name the expected digest and fetched digest.

#### Scenario: copied-source claims stay bounded

GIVEN a witness rebuild does not use `--require-independent-source`
WHEN the result is summarized
THEN the claim MUST remain bounded to rebuilding from the copied release request source archive
AND it MUST NOT claim independent source acquisition.
