## ADDED Requirements

### Requirement: Build finalization persists artifact attestations

The build pipeline MUST materialize native artifact attestations during build
finalization after final output identity, runtime references, and content
hashes are known.

Artifact attestation persistence MUST happen before the build is reported as a
successful outcome to the caller.

#### Scenario: Successful build reports only after attestation persistence

- GIVEN a derivation whose outputs have been built and hashed successfully
- WHEN crunch finalizes the build result
- THEN it computes and persists artifact attestations for the successful outputs
- AND only then returns those outputs as successful build outcomes

### Requirement: Provenance generation uses native build facts

The build pipeline MUST derive observed provenance facts from crunch's own
native build and store data instead of rescanning exported filesystem trees as
a separate source of truth.

At minimum the pipeline MUST use derivation inputs, lockfile/project facts
when available, `PathInfo`, final output hashes, and recorded runtime
references as the observed-facts inputs to provenance generation.

#### Scenario: Intermediate output without exported disk path still gets provenance

- GIVEN an intermediate output that exists in castore and `PathInfo` but is not exported to the host filesystem
- WHEN crunch needs its artifact attestation for closure assembly
- THEN crunch derives the observed facts from native build/store data
- AND provenance generation does not require rescanning a host-visible output directory
