# Specification: Content-addressed input resolution

## ADDED Requirements

### Requirement: CA inputs are resolved before dispatch

r[mantle.ca_input_resolution.resolved_derivation] Before cache lookup and dispatch of a derivation with a content-addressed prerequisite, directly or through a previously resolved intermediate, Mantle MUST produce a resolved derivation by replacing each provisional input path with its admitted realized path in arguments and environment and by replacing each admitted input edge with that path as an input source. Resolution MUST be a deterministic pure function of the derivation and the admitted realisation facts.

#### Scenario: Derivation with a CA input is resolved

- GIVEN a derivation whose argument references the provisional path of a completed CA input
- WHEN the worker prepares it for dispatch
- THEN the dispatched derivation MUST reference the realized path
- AND it MUST list the realized path as an input source instead of the CA input edge

#### Scenario: Unrealized input

- GIVEN a derivation whose CA input has no admitted realisation
- WHEN resolution runs
- THEN it MUST fail with `ca-input-unrealized` and the derivation MUST NOT be dispatched

### Requirement: Resolved identity keys CA-dependent work

r[mantle.ca_input_resolution.resolved_identity] For derivations with a direct or transitive CA prerequisite, Mantle MUST use the resolved derivation's Mantle BLAKE3 identity for cache lookup, CA output mappings, and shared action-result identity. For derivations in a CA-independent subgraph, derivation paths, output paths, CA mappings, and action refs MUST remain unchanged.

#### Scenario: CA-independent input-addressed derivation keeps its identity

- GIVEN a recorded derivation-path, output-path, and action-ref golden for a derivation whose prerequisites are entirely input-addressed and CA-independent
- WHEN it is converted and prepared after this change
- THEN all three values MUST equal the golden

#### Scenario: Two unresolved forms share one resolved form

- GIVEN two dependents whose unresolved derivations differ only in the derivation path of a CA input that realized identical output
- WHEN both are resolved
- THEN they MUST have the same resolved identity

### Requirement: Resolved identity enables early cutoff

r[mantle.ca_input_resolution.early_cutoff] Mantle MUST reuse every direct and transitive dependent without executing it when admitted results exist for its resolved identity, even when a CA prerequisite's unresolved derivation changed since that result was produced.

#### Scenario: Unchanged CA output stops downstream work

- GIVEN a completed build of a CA dependency and its dependents
- AND a change to the dependency's derivation that leaves its output bytes unchanged
- WHEN Mantle builds the graph again
- THEN the dependency MUST rerun and every dependent MUST be reused with zero executions
- AND the report MUST record each dependent's resolved identity and reuse disposition

#### Scenario: An unchanged CA output stops a transitive input-addressed chain

- GIVEN a CA producer, an input-addressed child consuming it, and an input-addressed grandchild consuming that child
- WHEN the CA producer derivation changes but its admitted output remains identical
- THEN only the CA producer MUST rerun
- AND the child and grandchild MUST both reuse signed resolved identities without execution

### Requirement: Realisations are signed and trust-checked

r[mantle.ca_input_resolution.realisation_records] Mantle MUST persist each CA and CA-resolved intermediate realisation as a record binding resolved identity, output name, output path, and signer, signed with the build's signing key. It MUST admit local and shared realisations only under the trusted-key policy used for PathInfo, matching full key material, and MUST share them through existing substitution and action-result sources.

#### Scenario: Clean client reuses a shared realisation

- GIVEN a publisher's signed realisation and PathInfo for a resolved derivation
- WHEN a clean client that trusts the publisher's key builds the same graph
- THEN it MUST admit the realisation and reuse the output without executing the derivation

#### Scenario: Untrusted realisation

- GIVEN a realisation signed by a key the client does not trust
- WHEN the client evaluates reuse
- THEN it MUST reject the record with `ca-realisation-untrusted` and build the derivation

#### Scenario: Replaced intermediate record

- GIVEN a completed CA-resolved input-addressed child whose signed action-result record is replaced by an untrusted same-name signer
- WHEN Mantle builds the grandchild
- THEN the forged record MUST NOT authorize reuse or dispatch of the grandchild
- AND the failure MUST identify the untrusted realisation

### Requirement: Plans can reference CA unit outputs

r[mantle.ca_input_resolution.dynamic_plan_binding] A native dynamic-plan unit MUST be able to reference a content-addressed unit output through a unit-output placeholder. The worker MUST bind that placeholder from the realized path during resolution instead of rejecting the plan at registration.

#### Scenario: Placeholder on a CA unit output

- GIVEN a plan whose unit argument contains a placeholder for a content-addressed unit output
- WHEN the plan is registered and the referenced unit completes
- THEN the referencing unit MUST be dispatched with the realized path in that argument

### Requirement: Resolution fails closed

r[mantle.ca_input_resolution.negative_controls] Mantle MUST reject conflicting realisations for one resolved identity and output, unsigned or untrusted realisations, wrong-domain identities, and resolution beyond named limits, with stable reasons. A conflict MUST block reuse and MUST be recorded as nondeterminism evidence.

#### Scenario: Conflicting realisations

- GIVEN two admissible realisations for one resolved identity and output with different output paths
- WHEN Mantle evaluates reuse
- THEN it MUST reject reuse with `ca-realisation-conflict` and MUST NOT dispatch a dependent from either ambiguous realisation
- AND the report MUST record both candidates
