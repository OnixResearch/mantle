## MODIFIED Requirements

### Requirement: Deterministic proof runs use an enforced sandbox envelope

The proof sandbox envelope requirement is extended with regression evidence: Mantle MUST keep tests or equivalent deterministic checks that prove proof-run sandbox invocations deny undeclared host access, do not expose the main rebuild output/store as writable proof-run inputs, and keep network access disabled by default.

#### Scenario: Isolation regression blocks undeclared host dependency

- GIVEN a deterministic proof rebuild recipe that can only succeed by reading an undeclared host path
- WHEN the deterministic proof run executes under the proof sandbox isolation regression harness
- THEN the run fails before a deterministic proof receipt is written
- AND the diagnostic identifies the undeclared host access or sandbox isolation denial

#### Scenario: Proof sandbox invocation excludes main output reuse

- GIVEN `release reproduce` has a main rebuild output directory and separate deterministic proof run directories
- WHEN Mantle constructs the proof sandbox command for a deterministic proof run
- THEN the sandbox invocation binds only the per-run output directory and per-run proof store as writable proof inputs
- AND it does not bind the main rebuild output directory or another run's proof store as a proof-run input

#### Scenario: Proof sandbox invocation denies network by default

- GIVEN deterministic proof runs are requested for a normal release reproducibility recipe
- WHEN Mantle constructs the proof sandbox command
- THEN the sandbox invocation uses the configured no-network proof profile
- AND regression evidence fails if the invocation opts into host networking without an explicit future spec change
