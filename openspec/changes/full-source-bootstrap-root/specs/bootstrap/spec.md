## ADDED Requirements

### Requirement: Full-source bootstrap root manifest

Crunch MUST define a versioned full-source bootstrap root manifest that names every source artifact, patch, digest, extraction rule, and expected provider output needed before the normalized seed contract is available.
ID: bootstrap.fullsource.root.manifest

The manifest MUST use BLAKE3 for crunch-owned artifact digests unless an upstream archive format or interoperability check requires another algorithm. Any non-BLAKE3 digest MUST name the reason. The manifest MUST fail validation if an artifact, patch, output, or trust note is missing a digest or provenance field.

#### Scenario: Valid root manifest is accepted

- GIVEN a manifest with version, source artifacts, patches, BLAKE3 digests,
  extraction rules, declared trust notes, and expected provider outputs
- WHEN the manifest validator runs
- THEN validation succeeds
- AND the output lists the exact normalized seed contract outputs it expects

#### Scenario: Missing source digest is rejected

- GIVEN a manifest source artifact without a digest
- WHEN the manifest validator runs
- THEN validation fails
- AND the diagnostic names the artifact and the missing digest field

#### Scenario: Non-BLAKE3 digest requires a reason

- GIVEN a manifest source artifact with a SHA-256 digest
- WHEN the manifest omits the interoperability reason
- THEN validation fails
- AND the diagnostic says why crunch-owned manifests default to BLAKE3

### Requirement: Source-built provider satisfies normalized seed contract

Crunch MUST support a source-built bootstrap provider that satisfies the existing normalized `bootstrap/seed.ncl` contract without deriving from the current musl.cc binary toolchain tarball.
ID: bootstrap.fullsource.provider.contract

The source-built provider MUST expose the same contract fields later bootstrap stages consume today: target-prefixed tool paths, headers, libraries, retained-tool metadata, reduction metadata, and provider notes. Later bootstrap derivations MUST keep depending on the normalized contract instead of provider-specific raw layouts.

#### Scenario: Source-built provider feeds make

- GIVEN a valid full-source root manifest and a source-built provider output
- WHEN `crunch build bootstrap/make.ncl` consumes that provider through
  `bootstrap/seed.ncl`
- THEN `make` builds successfully
- AND no later bootstrap derivation reads provider-specific raw paths

#### Scenario: Legacy fetched provider remains explicitly labeled

- GIVEN the operator selects the existing fetched provider path
- WHEN bootstrap docs or proof reports describe the run
- THEN they label it as seed-assisted legacy provider evidence
- AND they do not call it full-source bootstrap evidence

### Requirement: Full-source bootstrap claim requires evidence

Crunch MUST withhold the full-source bootstrap claim until a source-root manifest validates, the source-built provider satisfies the normalized seed contract, and a self-build proof completes with that provider.
ID: bootstrap.fullsource.claim.evidence

The claim evidence MUST include the manifest digest, provider output digest, proof bundle digest, and a docs update that separates remaining trusted roots from eliminated binary-provider trust. A prerequisite-only check MUST NOT count as full-source bootstrap evidence.

#### Scenario: Claim remains blocked before proof

- GIVEN the source-root manifest validates
- BUT no self-build proof has completed with the source-built provider
- WHEN docs or release evidence summarize bootstrap maturity
- THEN they keep the status below full-source bootstrap
- AND they name the missing proof evidence

#### Scenario: Successful proof promotes the claim

- GIVEN a valid source-root manifest
- AND a source-built provider satisfying the normalized seed contract
- AND a full self-build proof bundle produced with that provider
- WHEN bootstrap maturity is reported
- THEN the report may state full-source bootstrap root evidence exists
- AND it includes the manifest, provider, and proof bundle digests
