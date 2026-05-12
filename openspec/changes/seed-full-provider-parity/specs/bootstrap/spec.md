## MODIFIED Requirements

### Requirement: Bootstrap parity report gates provider rows by axis-specific evidence

Crunch MUST report normalized seed provider parity with enough granularity to avoid treating Guix source-root provider evidence as StageX-class lineage evidence.
ID: bootstrap.parity.provider.axis.evidence

The parity report MUST keep the `seed-full` row scoped to Guix/source-root provider contract evidence. It MUST report StageX-class normalized seed provider evidence as a separate row that remains blocked until lineage proof evidence is present. Completing the Guix `seed-full` row MUST NOT cause `--require stagex` to pass while StageX lineage, self-build, or other StageX blockers remain unresolved.

#### Scenario: Guix seed-full contract completes without StageX overclaim

- GIVEN `bootstrap/seed-full.ncl` exposes a normalized provider contract without legacy fetched-provider metadata
- WHEN `crunch bootstrap parity-report --json` runs
- THEN the `seed-full` row reports complete source-root provider evidence for the Guix axis
- AND a separate StageX seed-provider row remains blocking the StageX axis
- AND `crunch bootstrap parity-report --require stagex` exits non-zero while that StageX row is blocked

#### Scenario: Legacy seed-full metadata remains blocked

- GIVEN the seed-full derivation contains legacy fetched-provider raw metadata or omits normalized provider metadata
- WHEN `crunch bootstrap parity-report --json` runs
- THEN the Guix `seed-full` row blocks parity
- AND the diagnostic notes the missing or legacy provider contract evidence
