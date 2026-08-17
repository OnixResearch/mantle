## ADDED Requirements

### Requirement: Input trust policy offline proof rail emits bounded versioned evidence

r[project_workflows.input_trust_policy_proof_rail] Mantle MUST provide a bounded local offline proof rail that exercises input trust policy verification during refresh, accepts a refresh only when fetched bytes match the content hash and present valid trust evidence from the configured signer set, rejects before writing new lock entries on missing, malformed, invalid, untrusted, detached, or unsupported-verifier trust material, and emits a versioned, redacted, non-overclaiming evidence record that identifies the verified trust policy without exposing secret key material and states that a hash-only input is content integrity, not signer trust or upstream authenticity.

#### Scenario: trusted input refresh is accepted

GIVEN a project input declares a trust policy and the fetched bytes have matching content hash and valid trust evidence from the configured signer set
WHEN the rail refreshes the input
THEN Mantle MAY accept the lockfile update
AND the evidence MUST identify the verified trust policy (verifier kind and trusted key identity or fingerprint) without exposing secret key material.

#### Scenario: missing or invalid trust blocks the lock update

GIVEN a project input or patch requires trust evidence and the signature material is missing, malformed, invalid, from an untrusted key, from a same-name-but-different-material key, detached from the fetched bytes, or verified by an unsupported verifier
WHEN the rail refreshes the input
THEN Mantle MUST reject the refresh before writing new lockfile entries
AND existing lock entries MUST remain unchanged.

#### Scenario: hash-only input is not signed evidence

GIVEN an input has a content hash but no trust policy
WHEN the rail reports refresh or project soundness
THEN Mantle MAY claim content integrity against the recorded hash
AND it MUST NOT claim signer trust or upstream authenticity for that input, and the evidence MUST state the content-integrity-only non-claim.

#### Scenario: evidence is versioned redacted and non-overclaiming

GIVEN the rail emits its evidence record
WHEN the record is rendered
THEN it MUST carry a stable schema version, per-input trust decision, verified trust policy identity, and signature binding assertion
AND it MUST omit private key material, raw environment values, and unbounded logs and MUST NOT claim build success or release reproducibility from trust policy alone.
