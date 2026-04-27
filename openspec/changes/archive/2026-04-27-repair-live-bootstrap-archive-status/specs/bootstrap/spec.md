## MODIFIED Requirements

### Requirement: Full-source bootstrap claim requires evidence

Crunch MUST withhold the full-source bootstrap claim until a source-root manifest validates, the source-built provider satisfies the normalized seed contract, and a self-build proof completes with that provider.
ID: bootstrap.fullsource.claim.evidence

The claim evidence MUST include the provider kind, manifest digest, provider output digest, proof bundle digest, and a docs update that separates remaining trusted roots from eliminated binary-provider trust. A prerequisite-only check, placeholder derivation, deferred task, or archived partial-scaffolding change MUST NOT count as full-source bootstrap evidence. Self-build proof metadata MUST bind those digests to the selected source-built provider so a legacy fetched-provider run cannot satisfy the full-source claim.

#### Scenario: Claim remains blocked before proof

- GIVEN the source-root manifest validates
- BUT no self-build proof has completed with the source-built provider
- WHEN docs or release evidence summarize bootstrap maturity
- THEN they keep the status below full-source bootstrap
- AND they name the missing proof evidence

#### Scenario: Proof records selected source-built provider

- GIVEN `crunch self-build --no-substitute --source-root <manifest>` completes
- WHEN proof metadata is written
- THEN it records provider kind `source-root`, manifest digest, provider output digest, and proof bundle digest
- AND those fields are included in the proof bundle summary

#### Scenario: Successful proof promotes the claim

- GIVEN a valid source-root manifest
- AND a source-built provider satisfying the normalized seed contract
- AND a full self-build proof bundle produced with that provider
- WHEN bootstrap maturity is reported
- THEN the report may state full-source bootstrap root evidence exists
- AND it includes the manifest, provider, and proof bundle digests

#### Scenario: Deferred live-bootstrap archive is not completion evidence

- GIVEN a live-bootstrap archive contains deferred validation or placeholder derivations
- WHEN an operator checks whether the full-source bootstrap chain is complete
- THEN the archive is treated as partial scaffolding only
- AND full-source bootstrap status remains blocked until real build proof exists

#### Scenario: Deferred successor is not completion evidence

- GIVEN unfinished live-bootstrap work has been moved to an active successor change
- AND that successor still has unchecked implementation or proof tasks
- WHEN an operator checks whether the full-source bootstrap chain is complete
- THEN the deferral is treated as work tracking only
- AND full-source bootstrap status remains blocked until the successor records fresh proof transcripts

### Requirement: Legacy seed as development fast-path

The musl.cc-based seed MUST remain available as an opt-in development fast-path
while the full-source chain is being built out. Once the full chain passes all
validation (selftest, integration-test, self-build), the legacy seed MAY be
removed.
ID: bootstrap.legacy.seed.fastpath

#### Scenario: Developer uses legacy seed

- GIVEN a developer who does not want to wait for the full chain
- WHEN they build with the legacy seed option
- THEN the existing musl.cc tarball path is used and all downstream builds work

#### Scenario: Legacy seed selector is concrete

- GIVEN the full-source seed chain still contains placeholder derivations
- WHEN `bootstrap/seed.ncl` selects the legacy seed path
- THEN `bootstrap/seed-legacy.ncl` provides the concrete reduced musl.cc provider
- AND the legacy path does not import `seed-legacy.ncl` recursively
