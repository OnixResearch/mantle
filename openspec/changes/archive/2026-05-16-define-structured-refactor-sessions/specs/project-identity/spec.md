## ADDED Requirements

### Requirement: Project identity migrations SHOULD be represented as structured refactor sessions [r[project-identity.structured-refactor-sessions]]

Mantle MUST represent product identity migrations, including Crunch-to-Mantle compatibility, as structured refactor session records. A session record MUST name legacy aliases, canonical aliases, affected file names, affected CLI names, affected store-prefix semantics, compatibility policy, and validation checks.

#### Scenario: Crunch-to-Mantle session describes compatibility [r[project-identity.structured-refactor-sessions.crunch-mantle]]

- GIVEN the Crunch-to-Mantle migration session record
- WHEN an operator or test inspects it
- THEN it names `crunch` as a legacy command alias when retained
- AND it names `mantle` as the canonical command
- AND it records `/crunch/store` compatibility separately from `/mantle/store` defaults

#### Scenario: Mixed identity conflict is policy-backed [r[project-identity.structured-refactor-sessions.conflict]]

- GIVEN a project contains both canonical Mantle files and legacy Crunch files
- WHEN Mantle checks the project identity migration state
- THEN it emits a typed conflict diagnostic derived from the structured session policy
- AND it names the remediation action
