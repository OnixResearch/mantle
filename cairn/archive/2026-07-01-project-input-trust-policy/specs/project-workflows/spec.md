## ADDED Requirements

### Requirement: Project inputs and patches may require trust policy [r[project_workflows.input_trust_policy]]

Mantle MUST support explicit trust policy for project inputs and patch definitions. A trust policy MUST define supported verifier kind, signature material references, trusted public key identities or fingerprints, required signer or quorum decisions when supported, and binding to the fetched bytes or digest. Required trust policy MUST be verified before a lockfile refresh writes new source or patch hashes.

#### Scenario: Trusted input refresh is accepted [r[project_workflows.input_trust_policy.scenario.accept]]

- GIVEN a project input declares a trust policy and the fetched bytes have matching content hash and valid trust evidence from the configured signer set
- WHEN Mantle refreshes the input
- THEN Mantle MAY accept the lockfile update
- AND the refresh report MUST identify the verified trust policy without exposing secret key material.

#### Scenario: Missing or invalid trust blocks lock update [r[project_workflows.input_trust_policy.scenario.reject]]

- GIVEN a project input or patch requires trust evidence
- WHEN signature material is missing, malformed, invalid, from an untrusted key, detached from the fetched bytes, or verified by an unsupported verifier
- THEN Mantle MUST reject the refresh before writing new lockfile entries
- AND existing lock entries MUST remain unchanged.

#### Scenario: Hash-only input is not signed evidence [r[project_workflows.input_trust_policy.scenario.hash-only]]

- GIVEN an input has a content hash but no trust policy
- WHEN Mantle reports refresh or project soundness
- THEN Mantle MAY claim content integrity against the recorded hash
- AND it MUST NOT claim signer trust or upstream authenticity for that input.
