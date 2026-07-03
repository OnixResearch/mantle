# Verification Evidence Specification

## Purpose

Defines requirements for generating self-contained Nix-free demo evidence bundles from explicit proof inputs.

## Requirements

### Requirement: Nix-free demo bundle generator

r[verification_evidence.nix_free_demo_bundle_generator] Mantle SHOULD provide a generator that assembles a Nix-free demo bundle from explicit proof status, receipt digest, transcript, artifact digest, and non-claim inputs.

#### Scenario: Generator creates a valid bundle

GIVEN explicit proof metadata, transcripts, receipt digests, artifact digests, and non-claim text
WHEN the generator writes a demo bundle
THEN it MUST produce a `summary.json`, README, and supporting evidence files that pass the existing demo bundle validator
AND repeated generation from equivalent inputs MUST be deterministic.

#### Scenario: Generator does not run hidden proofs

GIVEN an operator asks the generator to package demo evidence
WHEN the command executes
THEN it MUST use only the explicit inputs supplied to it unless a proof-run option is separately requested
AND it MUST NOT claim proof execution from packaging alone.

### Requirement: Nix-free demo bundle manifest is complete

r[verification_evidence.nix_free_demo_bundle_manifest] Generated Nix-free demo bundles MUST include enough manifest data to audit status, command evidence, receipt digests, artifact digests, timestamps or run identifiers when provided, and bundle-local paths.

#### Scenario: Missing evidence is rejected

GIVEN a requested bundle omits a required transcript, digest, or proof status field
WHEN generation validates the manifest inputs
THEN it MUST fail with a deterministic diagnostic
AND it MUST NOT write a partial bundle that appears valid.

#### Scenario: Existing unrelated files are protected

GIVEN a destination directory contains unrelated files
WHEN generation would overwrite them
THEN the command MUST fail unless an explicit safe replacement policy is selected
AND it MUST report the conflicting path.

### Requirement: Nix-free demo generator preserves non-claims

r[verification_evidence.nix_free_demo_generator_non_claims] Generated Nix-free demo bundles MUST state whether evidence is successful, blocked, synthetic, partial, or demo-only, and MUST carry explicit non-claims when evidence is not a full proof.

#### Scenario: Blocked proof evidence stays blocked

GIVEN the input proof status is blocked before producing a stage binary
WHEN the generator writes the summary and README
THEN both outputs MUST identify the blocker and stage-binary absence
AND they MUST NOT claim Nix-free or fixed-point success.

#### Scenario: Synthetic evidence is labelled

GIVEN fixture or synthetic evidence is used for a demo bundle
WHEN the generator writes the bundle
THEN the summary and README MUST label the evidence as synthetic or fixture-derived
AND they MUST NOT present it as operator proof evidence.
