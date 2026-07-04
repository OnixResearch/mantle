## ADDED Requirements

### Requirement: No network during ordinary builds by default

r[build_correctness.no_network_by_default] Mantle MUST disable network access for ordinary derivation builds by default, keep network source acquisition inside declared fixed-output fetcher actions, and model any build-time network exception as an explicit audited sandbox capability.

#### Scenario: ordinary builder network attempt is blocked

GIVEN a normal derivation build has no declared network capability
WHEN the builder attempts to contact the network during sandbox execution
THEN Mantle MUST block the attempt or fail the build with a deterministic network-policy diagnostic
AND the output MUST NOT satisfy strong build-correctness evidence.

#### Scenario: fixed-output fetcher declares network input

GIVEN a builtin fetcher action declares URL, hash algorithm, expected digest, mode, and retry policy
WHEN Mantle acquires the source through that fetcher
THEN network access MAY occur only inside the fixed-output fetcher boundary
AND the admitted source MUST match the declared content hash before downstream builds use it.

#### Scenario: compatibility exception is scoped

GIVEN a foreign or compatibility build requests build-time network access
WHEN Mantle validates sandbox capability policy
THEN the exception MUST name the affected action, capability, policy basis, and audit class
AND an undeclared or policy-denied exception MUST fail closed before strong evidence is emitted.
