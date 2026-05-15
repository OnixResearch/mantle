## ADDED Requirements

### Requirement: Deterministic proof runs use an enforced sandbox envelope

The system MUST enforce a sandbox envelope for deterministic proof runs. When
an operator requests deterministic proof runs for release reproduction, Mantle
MUST execute each proof rebuild through that proof sandbox envelope instead of
directly spawning the rebuild command on the host.

The proof sandbox envelope MUST:

- mount the release evidence bundle read-only,
- mount the selected rebuild recipe/tool path read-only,
- mount only the per-run output directory and per-run proof store directory as
  writable paths,
- provide only the Mantle reproducibility environment variables plus an explicit
  minimal allowlist required to execute the recipe,
- deny network access by default,
- use a fresh writable output and proof-store directory for every run, and
- produce canonical profile evidence that can be digested and recorded in the
  deterministic proof receipt.

If deterministic proof runs are requested and the sandbox executor is missing,
unsupported on the current platform, or cannot enforce the requested network and
mount policy, Mantle MUST fail closed before writing a deterministic proof
receipt. Ordinary `release reproduce` without deterministic proof runs MAY remain
available.

#### Scenario: Deterministic proof run executes in sandbox

- GIVEN a release evidence bundle and a rebuild recipe that only needs the
  bundle, output directory, proof store directory, and declared environment
- WHEN `mantle release reproduce --deterministic-proof-runs 2` runs
- THEN each proof run executes through the proof sandbox envelope
- AND each run records the sandbox profile identity in the proof receipt

#### Scenario: Host-only recipe is not deterministic proof evidence

- GIVEN a rebuild recipe that succeeds only by reading an undeclared host path
- WHEN deterministic proof runs are requested
- THEN Mantle fails the deterministic proof run instead of producing a
  deterministic proof receipt
- AND the diagnostic identifies undeclared host access or sandbox execution
  failure as the blocker

#### Scenario: Missing sandbox executor fails closed

- GIVEN the current platform lacks a supported proof sandbox executor
- WHEN deterministic proof runs are requested
- THEN Mantle exits non-zero before writing a deterministic proof receipt
- AND the diagnostic says deterministic proof sandbox execution is unavailable
