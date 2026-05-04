## ADDED Requirements

### Requirement: patch 2.5.9 runtime validation waits for prerequisite execution proof [r[bootstrap.part.patch.2.5.9.runtime-validation]]
The system MUST keep patch 2.5.9 runtime proof incomplete until prerequisite make/tcc execution blockers are resolved and the produced patch binary applies a simple diff successfully.

#### Scenario: Version output alone is insufficient [r[bootstrap.part.patch.2.5.9.runtime-validation.version-insufficient]]
- **GIVEN** the produced patch binary reports its version
- **WHEN** applying a simple unified diff fails or is not tested
- **THEN** runtime validation remains incomplete

#### Scenario: Patch application is proven [r[bootstrap.part.patch.2.5.9.runtime-validation.diff-application]]
- **GIVEN** prerequisite runtime blockers are resolved
- **WHEN** `bootstrap/patch-tcc.ncl` builds and applies a simple unified diff successfully
- **THEN** the evidence records output path, smoke results, fallback status, and leakage-scan results
