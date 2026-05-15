## ADDED Requirements

### Requirement: Reproducibility proof report

Mantle MUST define a canonical reproducibility proof report that records enough evidence to justify a bounded reproducibility claim for release artifacts.
ID: release.verification.tech.reproducibility.proof

The report MUST record a closed proof class, release identifier, source digest, recipe identity, environment assumptions, clean rebuild store identities when a rebuild is claimed, produced output digest set, comparison verdict, and the evidence artifact digests that were checked before the verdict was emitted. Mantle-owned output comparisons MUST use BLAKE3 unless the report also records an explicit interoperability reason for a non-BLAKE3 digest. The first supported proof classes MUST extend the existing technical release classes and distinguish at least:

- `bundle-consistent`: the release-evidence bundle is internally valid, but no self-proof or rebuild agreement is claimed.
- `self-proof-valid`: the bundled self-proof verifies for the published digest set, but no additional clean rebuild agreement is claimed.
- `self-rebuild-match`: Mantle rebuilt the release artifact from the recorded recipe in a clean local store and matched the published BLAKE3 output digest set.
- `external-witness-match`: at least one accepted witness rebuilt from the exported request and matched the published BLAKE3 output digest set.
- `policy-satisfied`: the accepted witness set satisfies the configured social verification policy.

A report MUST NOT use a stronger proof class than the evidence supports. A mismatch, unsupported recipe, dirty input, missing required digest, reused non-clean rebuild store where a clean store is required, or incomplete witness material MUST produce a failed report or a weaker class rather than silently updating the published digest set. Existing `self-proof-valid` release verification behavior MUST remain valid and MUST NOT be renamed away by the reproducibility report work.

#### Scenario: Local clean rebuild proves self rebuild match

- GIVEN a release-evidence bundle that verifies successfully
- AND the bundle records a supported rebuild recipe identity
- AND Mantle rebuilds the release artifact in a clean local store
- WHEN the rebuilt output BLAKE3 digest set equals the published digest set
- THEN the reproducibility report records proof class `self-rebuild-match`
- AND it records the recipe identity, clean store identity, source digest, published digest set, rebuilt digest set, and comparison verdict

#### Scenario: Independent witness promotes the proof class

- GIVEN a release-evidence bundle that verifies successfully
- AND an accepted witness attestation references the release attestation
- AND the witness rebuilt output BLAKE3 digest set equals the published digest set
- WHEN release verification evaluates the witness material
- THEN the reproducibility report may record proof class `external-witness-match`
- AND it MUST NOT record `policy-satisfied` unless the configured witness policy is also satisfied

#### Scenario: Mismatch fails closed

- GIVEN a clean rebuild or witness rebuild produces a different output BLAKE3 digest set
- WHEN Mantle creates or verifies the reproducibility report
- THEN the report records a failed comparison verdict naming the mismatched output
- AND it does not replace the published digest set with the rebuilt digest set
- AND it does not claim `self-rebuild-match`, `external-witness-match`, or `policy-satisfied`

#### Scenario: Bundle consistency does not over-claim reproducibility

- GIVEN a release-evidence bundle verifies internally
- BUT no clean rebuild or accepted witness rebuild evidence is present
- WHEN Mantle reports the technical proof class
- THEN it may report `bundle-consistent`
- AND it does not claim any rebuild-based reproducibility class

### Requirement: Reproducibility proof recipe is replayable

Mantle MUST make the rebuild recipe used for a reproducibility proof replayable by another environment without inventing ad hoc commands.
ID: release.verification.tech.reproducibility.recipe

The recipe identity MUST bind the command, workflow version, source/bundle inputs, output selection, sandbox expectations, and required environment assumptions. A reproducibility report MUST reject unknown recipe identities unless a compatible implementation explicitly supports them. The first supported recipe MAY be the existing self-hosting proof workflow and exported witness request workflow, but any accepted recipe MUST be versioned and documented.

#### Scenario: Supported recipe can be replayed from evidence

- GIVEN a reproducibility request exported from a verified release-evidence bundle
- AND the request names a supported recipe identity
- WHEN another environment runs the documented witness rebuild entry point
- THEN the rebuilt outputs are selected by the recipe rather than by operator guesswork
- AND the resulting witness material records the replayed recipe identity

#### Scenario: Unknown recipe is rejected

- GIVEN a release-evidence bundle or witness request names an unsupported recipe identity
- WHEN Mantle attempts to create or verify reproducibility evidence
- THEN the command exits non-zero
- AND the diagnostic names the unsupported recipe identity

### Requirement: Reproducibility claim wording remains bounded

Mantle MUST keep docs and machine-readable reports clear about what each reproducibility proof class proves and what it does not prove.
ID: release.verification.tech.reproducibility.claims

Docs and reports MUST distinguish release-artifact reproducibility from full-source bootstrap, cross-platform determinism, global reproducibility, and social verification sufficiency. They MUST name environmental assumptions such as host kernel, CPU architecture, sandbox implementation, and filesystem behavior when those assumptions are outside the proof boundary.

#### Scenario: Docs separate artifact reproducibility from bootstrap claims

- GIVEN a contributor reads the release reproducibility documentation
- WHEN a report reaches `self-rebuild-match` or `external-witness-match`
- THEN the docs explain that the named release artifacts reproduced under recorded assumptions
- AND they do not claim that this alone proves full-source bootstrap or global reproducibility

#### Scenario: Report records assumptions outside the proof boundary

- GIVEN a reproducibility report is emitted
- WHEN it lists its proof boundary
- THEN host/kernel/CPU/sandbox assumptions are recorded as assumptions
- AND they are not counted as source-built inputs or independent witnesses
