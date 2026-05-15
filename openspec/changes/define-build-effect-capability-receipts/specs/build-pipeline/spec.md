## ADDED Requirements

### Requirement: Build execution MUST expose auditable effect observations [r[build-pipeline.effect-observations]]

Mantle build execution MUST produce auditable effect observations for proof-capable runs. Observations MUST be expressed with the active Mantle build-effect policy and MUST at minimum distinguish local store reads, output writes, environment access, host-tool access, network access, clock access, randomness, secret access, and remote-build dispatch when those effects are observable by the selected sandbox/audit profile.

#### Scenario: Local sandbox run emits local effects [r[build-pipeline.effect-observations.local]]

- GIVEN a proof-capable local sandbox build that only reads declared store inputs and writes declared outputs
- WHEN the build completes
- THEN the effect observation stream includes local store-read and output-write effects
- AND it does not include network, secret, or remote-build effects

#### Scenario: Incomplete audit is explicit [r[build-pipeline.effect-observations.incomplete]]

- GIVEN the selected execution profile cannot observe one or more required effect families
- WHEN a deterministic proof-capable build is requested
- THEN Mantle records the missing observation coverage
- AND deterministic proof verification treats the run as ineligible unless policy explicitly allows that gap
