## ADDED Requirements

### Requirement: Release build can require Nix cross-builder witness

The release build pipeline MUST provide an explicit release-only path to produce and require a Nix cross-builder witness for the selected Mantle release artifact.
ID: build-pipeline.release.nix-cross-builder-witness

When enabled, the release build MUST build or locate the Mantle self-built release artifact, build the corresponding Nix artifact from the same recorded source and build policy, compare the selected artifact bytes with BLAKE3, write a `mantle-nix-cross-builder-witness-v1` receipt, and fail the release artifact when the operator requested the witness as required and the receipt verdict is not `nix-witness-match`.

Ordinary developer builds MUST NOT be required to run the Nix cross-builder witness. The heavy check SHOULD be exposed through a release package/check or a release command flag rather than hidden in ad hoc shell scripts.

#### Scenario: Required Nix witness gates the release artifact

- GIVEN an operator invokes the release build with Nix cross-builder witness required
- AND the Mantle deterministic proof receipt validates as `self-rebuild-match`
- WHEN the Nix-built selected artifact and Mantle self-built selected artifact have identical BLAKE3 digest sets
- THEN the release build writes the Nix witness receipt into the release evidence output
- AND the release artifact may complete

#### Scenario: Required Nix witness mismatch fails the release artifact

- GIVEN an operator invokes the release build with Nix cross-builder witness required
- AND the Nix-built selected artifact differs from the Mantle self-built selected artifact
- WHEN the comparison finishes
- THEN the release build writes or reports a `cross-builder-mismatch` verdict
- AND the required release build exits non-zero before publishing a successful release artifact

#### Scenario: Non-release builds stay fast

- GIVEN a developer builds the default Mantle package for local iteration
- WHEN no release witness option is selected
- THEN the build does not run the Nix cross-builder witness rail
- AND determinism or cross-builder absence is not silently reported as success
