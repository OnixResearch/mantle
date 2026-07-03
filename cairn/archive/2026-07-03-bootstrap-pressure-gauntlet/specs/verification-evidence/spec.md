## ADDED Requirements

### Requirement: Bootstrap pressure gauntlet

r[verification_evidence.bootstrap_pressure_gauntlet] Mantle MUST track bootstrap-strengthening profiles with explicit trust roots, protected-exec evidence, fixed-point status, and remaining blockers before claiming stronger bootstrap or reproducibility properties.

#### Scenario: proof profile ladder is closed

GIVEN Mantle runs bootstrap pressure profiles such as default, non-Nix-host, no-host-tools, source-built-provider, reduced-seed, or full-source-root attempt
WHEN it emits a bootstrap pressure report
THEN the report MUST classify the profile with a stable verdict and bind seed inventory digest, protected-exec audit digest when applicable, stage output digest set, fixed-point status, and host-tool availability policy
AND intermediate profiles MUST NOT be described as full-source bootstrap roots.

#### Scenario: undeclared host execution fails closed

GIVEN a no-host-tools or protected-exec bootstrap profile observes an undeclared compiler, build tool, archive tool, Nix command, shell, or helper executable
WHEN Mantle evaluates the protected phase
THEN the profile MUST fail closed before stronger bootstrap evidence is emitted
AND the report MUST identify the missing inventory entry or forbidden execution class.

#### Scenario: remaining seed trust is explicit

GIVEN a bootstrap pressure run still depends on a binary seed, fetched artifact, host prerequisite, or unsupported source-built stage
WHEN Mantle summarizes bootstrap status
THEN the report MUST list the remaining trusted root and next blocker
AND release/global reproducibility summaries MUST NOT turn that partial proof into compiler correctness or full-source bootstrap correctness.
