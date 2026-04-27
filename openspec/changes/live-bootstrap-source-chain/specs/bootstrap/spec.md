## ADDED Requirements

### Requirement: Source-built bootstrap chain implementation

Crunch MUST implement the live-bootstrap stage chain through a source-built provider that satisfies the normalized seed contract without using the legacy musl.cc binary provider.
ID: bootstrap.source.chain.implementation

The chain MUST replace `bootstrap/binutils-tcc.ncl`, `bootstrap/gcc-4.0.ncl`, `bootstrap/gcc-4.7.ncl`, `bootstrap/gcc-10.ncl`, `bootstrap/musl-full.ncl`, `bootstrap/binutils-full.ncl`, and `bootstrap/seed-full.ncl` placeholder derivations before they can satisfy bootstrap completion status. Stage validation MUST record command transcripts for each transition stage, `bootstrap/selftest.ncl`, `bootstrap/integration-test.ncl`, and final self-build proof. Validation MUST fail closed when a stage emits placeholder text, exits through a deferred task, or falls back to the legacy provider. The final provider MUST expose the normalized seed contract fields consumed by later bootstrap derivations.

#### Scenario: Stage placeholder is rejected

- GIVEN a bootstrap stage emits `ERROR: ... is a placeholder`
- WHEN source-chain validation evaluates completion status
- THEN the stage is reported incomplete
- AND full-source bootstrap status remains blocked

#### Scenario: Stage chain validates in order

- GIVEN each source-chain derivation has been implemented
- WHEN validation runs stage-by-stage from `stage0-posix` through `seed-full`
- THEN every stage transcript records the command, provider selection, exit status, and output path
- AND later stages consume only the normalized provider contract

#### Scenario: Final proof binds provider evidence

- GIVEN `bootstrap/seed-full.ncl` satisfies the normalized provider contract
- WHEN `crunch self-build` completes with the source-built provider
- THEN proof metadata records provider kind, manifest digest, provider output digest, and proof bundle digest
- AND docs separate remaining trust roots from eliminated binary-provider trust

#### Scenario: Final bootstrap tests use source-built provider

- GIVEN the source-built provider satisfies the normalized seed contract
- WHEN `bootstrap/selftest.ncl` and `bootstrap/integration-test.ncl` are built
- THEN both transcripts record source-built provider selection
- AND neither transcript uses the legacy musl.cc provider

## MODIFIED Requirements

### Requirement: Full-source bootstrap claim requires evidence

Crunch MUST withhold the full-source bootstrap claim until every named live-bootstrap placeholder is replaced, source-built stage transcripts exist, and self-build proof completes with the source-built provider.
ID: bootstrap.fullsource.claim.evidence

The claim evidence MUST include provider kind, manifest digest, provider output digest, proof bundle digest, stage-by-stage build transcripts through `bootstrap/seed-full.ncl`, `bootstrap/selftest.ncl` and `bootstrap/integration-test.ncl` transcripts, and a docs update that separates remaining trusted roots from eliminated binary-provider trust. A prerequisite-only check, placeholder derivation, deferred task, archived partial-scaffolding change, or unfinished successor task MUST NOT count as full-source bootstrap evidence.

#### Scenario: Placeholder blocks full-source claim

- GIVEN any of `bootstrap/binutils-tcc.ncl`, `bootstrap/gcc-4.0.ncl`, `bootstrap/gcc-4.7.ncl`, `bootstrap/gcc-10.ncl`, `bootstrap/musl-full.ncl`, `bootstrap/binutils-full.ncl`, or `bootstrap/seed-full.ncl` still emits placeholder text
- WHEN bootstrap maturity is reported
- THEN full-source bootstrap evidence remains absent
- AND the report names the unresolved placeholder stage

#### Scenario: Final source proof records docs separation

- GIVEN all stage transcripts and self-build proof complete with the source-built provider
- WHEN bootstrap maturity docs are updated
- THEN the docs name remaining trust roots and eliminated binary-provider trust separately
- AND the proof bundle digest is recorded next to provider kind, manifest digest, and provider output digest

### Requirement: StageX-class self-build proof binds lineage evidence

Crunch MUST require the full StageX-class evidence tuple before this live-bootstrap source-chain change can satisfy any StageX-class bootstrap claim.
ID: bootstrap.stagex.selfbuild.proof

The proof metadata MUST bind audited seed digest, lineage manifest digest, stage graph digest, normalized provider digest, staged source digest, stage1 and stage2 crunch binary digests, bootstrap-tool digests, protected execution audit digest when used, final proof bundle digest, and canonical reproducibility report digest. The proof MUST fail closed when any named live-bootstrap placeholder remains, when the legacy provider is selected, or when forbidden host executables run during the protected stage.

#### Scenario: StageX proof records complete evidence tuple

- GIVEN the live-bootstrap source chain materializes a normalized provider
- AND a full protected self-build proof completes with that provider
- WHEN StageX-class proof metadata is written
- THEN it records audited seed digest, lineage manifest digest, stage graph digest, normalized provider digest, staged source digest, stage1 digest, stage2 digest, bootstrap-tool digests, protected execution audit digest when used, final proof bundle digest, and canonical reproducibility report digest
- AND it records provider kind as StageX-class lineage

#### Scenario: StageX proof rejects legacy fallback

- GIVEN any stage selected the legacy musl.cc provider
- WHEN StageX-class evidence is requested
- THEN the proof fails closed
- AND no StageX-class claim is emitted
