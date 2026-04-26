## ADDED Requirements

### Requirement: Full-source bootstrap root manifest

Crunch MUST define a versioned full-source bootstrap root manifest that names every source artifact, patch, digest, extraction rule, and expected provider output needed before the normalized seed contract is available.
ID: bootstrap.fullsource.root.manifest

The manifest MUST use BLAKE3 for crunch-owned artifact digests unless an upstream archive format or interoperability check requires another algorithm. Any non-BLAKE3 digest MUST name the reason. The manifest MUST fail validation if an artifact, patch, output, network trust root, or trust note is missing a digest or provenance field. Any remaining tiny seed or bootstrap assumption MUST be represented as a trust note with digest, provenance, scope, and rationale; undeclared non-source inputs MUST fail validation.

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

#### Scenario: Undeclared patch is rejected

- GIVEN a source artifact references a patch name
- BUT the manifest patch list does not declare that patch with digest and provenance
- WHEN the manifest validator runs
- THEN validation fails
- AND the diagnostic names the undeclared patch

#### Scenario: Expected provider output metadata is required

- GIVEN a manifest expected provider output without digest or provenance
- WHEN the manifest validator runs
- THEN validation fails
- AND the diagnostic names the output and missing field

#### Scenario: Unexpected provider output is rejected

- GIVEN a source-built provider emits an output not listed by the manifest
- WHEN provider output validation runs
- THEN validation fails
- AND the diagnostic names the unexpected output

#### Scenario: Unmanifested network trust root is rejected

- GIVEN provider construction needs a network URL not present in manifest source artifacts or network trust roots
- WHEN provider dependency validation runs
- THEN validation fails before build execution
- AND the diagnostic names the unmanifested URL

#### Scenario: Trust note provenance is required

- GIVEN a remaining tiny seed or bootstrap assumption trust note
- WHEN the trust note omits digest, provenance, scope, or rationale
- THEN validation fails
- AND the diagnostic names the incomplete trust note

### Requirement: Source-built provider satisfies normalized seed contract

Crunch MUST support a source-built bootstrap provider that satisfies the existing normalized `bootstrap/seed.ncl` contract without deriving from the current musl.cc binary toolchain tarball.
ID: bootstrap.fullsource.provider.contract

The source-built provider MUST expose the same contract fields later bootstrap stages consume today: target-prefixed tool paths, headers, libraries, retained-tool metadata, reduction metadata, and provider notes. Later bootstrap derivations MUST keep depending on the normalized contract instead of provider-specific raw layouts. `crunch bootstrap --source-root <manifest>` MUST select the source-built provider; the existing `crunch bootstrap --fetch` path MUST remain the seed-assisted legacy provider; specifying both MUST fail before provider work starts.

#### Scenario: Source-built provider feeds make

- GIVEN a valid full-source root manifest and a source-built provider output
- WHEN `crunch build bootstrap/make.ncl` consumes that provider through
  `bootstrap/seed.ncl`
- THEN `make` builds successfully
- AND no later bootstrap derivation reads provider-specific raw paths

#### Scenario: Ambiguous provider selection fails closed

- GIVEN an operator passes both `--fetch` and `--source-root <manifest>`
- WHEN bootstrap provider selection runs
- THEN it exits non-zero before fetching or building provider inputs
- AND the diagnostic says the provider selection is ambiguous

#### Scenario: Legacy fetched provider remains explicitly labeled

- GIVEN the operator selects the existing fetched provider path
- WHEN bootstrap docs or proof reports describe the run
- THEN they label it as seed-assisted legacy provider evidence
- AND they do not call it full-source bootstrap evidence

### Requirement: Full-source bootstrap claim requires evidence

Crunch MUST withhold the full-source bootstrap claim until a source-root manifest validates, the source-built provider satisfies the normalized seed contract, and a self-build proof completes with that provider.
ID: bootstrap.fullsource.claim.evidence

The claim evidence MUST include the provider kind, manifest digest, provider output digest, proof bundle digest, and a docs update that separates remaining trusted roots from eliminated binary-provider trust. A prerequisite-only check MUST NOT count as full-source bootstrap evidence. Self-build proof metadata MUST bind those digests to the selected source-built provider so a legacy fetched-provider run cannot satisfy the full-source claim.

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
