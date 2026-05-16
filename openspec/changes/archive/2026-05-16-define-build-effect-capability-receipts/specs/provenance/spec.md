## ADDED Requirements

### Requirement: Provenance MUST preserve declared effects and observed effects separately [r[provenance.effect-claims-and-facts]]

Mantle provenance records MUST represent declared build/proof effects as claims and observed build/proof effects as facts. Canonical attestation digests MUST include both sections when present so that changing the declared or observed effect set changes the provenance identity.

#### Scenario: Declared and observed effects are not conflated [r[provenance.effect-claims-and-facts.separate]]

- GIVEN a recipe declares no network effect
- AND the build audit observes no network effect
- WHEN Mantle materializes provenance for the output
- THEN the attestation records the declared effect claims separately from observed effect facts
- AND downstream queries can compare the two sets without parsing prose
