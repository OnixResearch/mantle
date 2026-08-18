## ADDED Requirements

### Requirement: Build correctness receipts [r[verification_evidence.build_correctness_receipts]]

Mantle MUST emit deterministic receipts for action-correct build claims. A receipt MUST bind action ref, Nickel evaluation receipt ref when the action was produced from Mantle `.ncl`, input object refs, toolchain refs, produced object refs, reference scan ref, sandbox report ref, network policy result, producer identity, signature refs when present, execution status, and build-or-reuse reason. Human and JSON output MUST keep claims bounded to the exact action/object evidence present.

#### Scenario: Successful action receipt is complete [r[verification_evidence.build_correctness_receipts.scenario.success]]

- GIVEN Mantle executes an action under enforced policy and admits produced CAS objects
- WHEN it emits a build correctness receipt
- THEN the receipt MUST include action ref, Nickel evaluation receipt ref when applicable, input refs, toolchain refs, produced object refs, reference scan ref, sandbox report ref, producer identity, and execution status
- AND the receipt MUST be deterministic for equivalent declared inputs and outputs

#### Scenario: Reuse receipt names trust basis [r[verification_evidence.build_correctness_receipts.scenario.reuse]]

- GIVEN Mantle accepts a reused or substituted output
- WHEN it emits a build correctness receipt
- THEN the receipt MUST name the prior receipt or signature material that justified reuse
- AND the receipt MUST explain the matched action ref, object refs, policy, and producer trust basis

#### Scenario: Evidence does not overclaim [r[verification_evidence.build_correctness_receipts.scenario.non-goals]]

- GIVEN Mantle emits a successful action-correct receipt
- WHEN human or JSON evidence is rendered
- THEN the evidence MAY claim the produced objects match the declared action and receipt policy
- AND it MUST NOT claim compiler correctness, source-to-binary reproducibility, full bootstrap correctness, frontend module correctness, deploy success, or physical-target determinism unless separate evidence exists

#### Scenario: Secret material is redacted [r[verification_evidence.build_correctness_receipts.scenario.secret-redaction]]

- GIVEN a build action or output involves secret descriptors
- WHEN Mantle emits receipts, reports, logs, or diagnostics
- THEN it MUST include only redacted descriptors or encrypted/object refs
- AND it MUST NOT include decrypted secret bytes or inline plaintext secret content
