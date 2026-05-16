## ADDED Requirements

### Requirement: Remote realization MUST use hash-negotiated content transfer [r[distributed-builds.hash-negotiated-realization]]

Mantle distributed realization MUST define a provider-neutral, versioned handshake in which the scheduler first sends realization identity, recipe identity, declared input identities, platform/profile facts, and declared execution capabilities. A worker MUST respond with either an unsupported-profile/capability denial or a missing-content set keyed by content kind and digest. Content transfer MUST verify each digest before the worker executes the realization.

#### Scenario: Worker already has all content [r[distributed-builds.hash-negotiated-realization.all-present]]

- GIVEN a worker already has the requested recipe and input content identities
- WHEN the scheduler sends a remote realization request
- THEN the worker returns an empty missing-content set
- AND execution may proceed without transferring duplicate content

#### Scenario: Worker requests missing content by digest [r[distributed-builds.hash-negotiated-realization.missing-set]]

- GIVEN a worker lacks one declared source tree and one recipe dependency
- WHEN it evaluates the remote realization request
- THEN it returns a missing-content set naming each missing item by kind and digest
- AND the scheduler transfers only those requested items

#### Scenario: Digest mismatch aborts before execution [r[distributed-builds.hash-negotiated-realization.digest-mismatch]]

- GIVEN transferred content does not match the requested digest
- WHEN the worker verifies the transfer
- THEN the worker aborts before executing the builder
- AND the returned result classifies the provider evidence as invalid

### Requirement: Remote realization receipts MUST bind negotiated inputs and outputs [r[distributed-builds.hash-negotiated-receipts]]

Remote realization receipts MUST record the handshake version, realization key, worker profile, negotiated input digest set, declared capability set, observed effect/capability summary when available, and produced output digest set. Receipts MUST be rejected as deterministic proof input if negotiated input identities or output identities are missing.

#### Scenario: Receipt records negotiated inputs [r[distributed-builds.hash-negotiated-receipts.inputs]]

- GIVEN a remote worker completes a realization after missing-content negotiation
- WHEN it returns a receipt
- THEN the receipt records every negotiated input digest accepted by the worker
- AND it records every produced output digest returned to the scheduler
