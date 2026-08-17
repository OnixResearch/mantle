## ADDED Requirements

### Requirement: Adversarial hermeticity gauntlet

r[verification_evidence.adversarial_hermeticity_gauntlet] Mantle MUST exercise hermeticity claims with declared adversarial perturbations and fail closed or record blockers when hidden host influence is observed.

#### Scenario: perturbation profile is declared

GIVEN an adversarial hermeticity gauntlet run is requested
WHEN Mantle creates the gauntlet plan
THEN the plan MUST bind the selected host-tool, environment, network, timestamp, locale, umask, temp-path, store-path, and randomness perturbation axes
AND unsupported axes MUST be recorded as explicit non-claims.

#### Scenario: strict mode fails closed

GIVEN a strict hermeticity cell observes undeclared host execution, network access, ambient environment leakage, undeclared store references, or nondeterministic path dependence
WHEN Mantle emits evidence for that cell
THEN the cell MUST fail closed or carry a strict blocker before release/global reproducibility evidence can be admitted
AND the diagnostic MUST identify the violated hermeticity class.

#### Scenario: practical mode degradation cannot satisfy strict evidence

GIVEN a practical-mode cell continues after a degraded hermeticity event
WHEN Mantle summarizes the gauntlet result
THEN the report MAY preserve the build output and audit event for diagnostics
AND it MUST NOT use that cell as strict reproducibility evidence.
