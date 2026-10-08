# Specification: Dynamic-plan output inputs

## ADDED Requirements

### Requirement: Static derivations can reference plan roots

r[mantle.dynamic_plan_output_inputs.typed_reference] Mantle MUST provide a typed Nickel input that references one output of one root unit of one declared dynamic-plan output of a producer derivation. Evaluation MUST reject a reference whose plan output is not declared in the producer's `dynamic_plan_outputs` and a placeholder marker that names no such input.

#### Scenario: Valid reference is converted

- GIVEN a producer derivation that declares `plan` in `dynamic_plan_outputs`
- AND a consumer input referencing plan output `plan`, root `unit.app`, and unit output `out`
- WHEN Mantle evaluates and converts the consumer
- THEN the consumer derivation MUST record an input edge on the producer's `plan` output and one canonical binding record

#### Scenario: Undeclared plan output

- GIVEN a consumer input that references plan output `out` on a producer that declares only `plan`
- WHEN Mantle evaluates the consumer
- THEN evaluation MUST fail with an error naming the undeclared output

### Requirement: Consumers have evaluation-time identity

r[mantle.dynamic_plan_output_inputs.request_identity] A consumer's derivation path and input-addressed output paths MUST be computable at evaluation from the producer derivation identity, the plan output name, the root unit id, the unit output name, and the consumer's other inputs. Mantle MUST NOT change the derivation format to represent the reference.

#### Scenario: Stable identity across evaluations

- GIVEN two evaluations of the same consumer with the same producer derivation and reference names
- WHEN Mantle converts both
- THEN both MUST produce the same consumer derivation path and output paths

#### Scenario: Different root changes identity

- GIVEN two consumers identical except for the referenced root unit id
- WHEN Mantle converts both
- THEN their derivation paths MUST differ

### Requirement: References are bound at dispatch

r[mantle.dynamic_plan_output_inputs.dispatch_binding] When a consumer is dispatched, the worker MUST replace every placeholder for a reference with the bound root unit output path, MUST mount that path and its reference closure in the sandbox, and MUST include it in reference scanning. A consumer MUST NOT be dispatched while any reference is unbound.

#### Scenario: Builder reads the root output

- GIVEN a consumer whose argument contains the placeholder for a root output
- WHEN the root has completed and the consumer is dispatched
- THEN the builder MUST receive the root's output path in that argument and MUST be able to read it in the sandbox
- AND the consumer's PathInfo MUST list the root output as a reference when the consumer output embeds that path

#### Scenario: Content-addressed root

- GIVEN a referenced root unit with a content-addressed output
- WHEN the consumer is dispatched after the root completes
- THEN the placeholder MUST be replaced with the root's realized output path

### Requirement: Binding failures are typed

r[mantle.dynamic_plan_output_inputs.binding_failures] The worker MUST bind references after the producer's plan is accepted and MUST fail the consumer with a stable reason when the producer fails, the plan is rejected, the root is absent from the plan's roots, the root lacks the output, the root fails, or the per-consumer reference limit is exceeded. A failed consumer MUST NOT record a successful output.

#### Scenario: Root missing from plan

- GIVEN an accepted plan whose roots do not include the referenced unit
- WHEN the worker binds the consumer
- THEN the consumer MUST fail with `plan-output-root-missing`
- AND its dependents MUST fail through the existing failure path

#### Scenario: Plan rejected

- GIVEN a producer whose declared plan output fails plan validation
- WHEN the worker handles the producer's completion
- THEN every consumer referencing that plan output MUST fail with `plan-output-plan-rejected`

### Requirement: Bindings are reported

r[mantle.dynamic_plan_output_inputs.provenance] Build reports and provenance records MUST record, for every bound reference, the producer derivation path, plan output name, canonical plan digest, root unit id, unit output name, bound unit derivation path, and bound output path, in canonical order.

#### Scenario: Successful binding is reported

- GIVEN a consumer that built with one bound reference
- WHEN the build report is written
- THEN the consumer's row MUST include the reference, the plan digest, and both bound paths

#### Scenario: Failed binding is reported

- GIVEN a consumer that failed with `plan-output-output-missing`
- WHEN the build report is written
- THEN the row MUST include the reference and the failure reason
- AND it MUST NOT include a bound output path
