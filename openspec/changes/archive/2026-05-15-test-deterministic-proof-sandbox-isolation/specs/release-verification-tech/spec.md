## MODIFIED Requirements

### Requirement: Release verification consumes deterministic proof receipts without overclaiming

Deterministic-release eligibility is strengthened by requiring maintained isolation-regression evidence for the supported sandbox profile family. A supported-looking sandbox profile identity MUST NOT be treated as sufficient if Mantle's proof-run executor path can be configured or regressed to expose undeclared host paths, host networking, the main rebuild output, or reused proof stores without failing closed.

#### Scenario: Supported profile requires isolation-negative evidence

- GIVEN a deterministic-build proof receipt records a supported `mantle-proof-sandbox-v1:` profile identity
- AND Mantle's maintained regression evidence shows that profile family denies undeclared host access, host networking by default, and main-output/proof-store reuse
- WHEN release verification evaluates deterministic claim eligibility
- THEN the supported profile identity may contribute to deterministic-release eligibility

#### Scenario: Isolation regression failure blocks deterministic promotion

- GIVEN deterministic proof output digests match the release artifacts
- BUT the sandbox isolation regression for that profile family fails or is bypassed
- WHEN release verification evaluates deterministic claim eligibility
- THEN the release remains at the strongest lower satisfied proof class
- AND the report identifies unsupported deterministic proof sandbox isolation evidence as the blocker
