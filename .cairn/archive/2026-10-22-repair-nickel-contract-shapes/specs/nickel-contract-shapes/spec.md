# Specification: Nickel standard-library shape requirements

## ADDED Requirements

### Requirement: Published plan shapes evaluate under the pinned interpreter

r[mantle.nickel_contract_shapes.evaluable_shapes] Every standard-library
module MUST evaluate under the pinned Nickel interpreter, and every published
plan shape MUST keep the field names, field order, default values,
requiredness, and closedness it declared before the repair. A shape with
several required fields MUST use a record type or a whole-shape check rather
than a record contract with more than one undefined field.

#### Scenario: A plan that omits a required field

- GIVEN a published plan shape with a required field and a defaulted field
- WHEN a value without the required field is evaluated against the shape
- THEN the evaluation MUST fail
- AND a value with the required field omitted but every other field present
  MUST still be rejected

#### Scenario: A plan that carries an undeclared field

- GIVEN a published plan shape
- WHEN a value with an undeclared field is evaluated against the shape
- THEN the evaluation MUST fail
- AND the shape MUST NOT accept the undeclared field

#### Scenario: A plan that omits a defaulted field

- GIVEN a published plan shape with a defaulted field
- WHEN a value without that field is evaluated against the shape
- THEN the evaluation MUST succeed
- AND the evaluated value MUST carry the declared default

### Requirement: The Rust layer stays the admission authority

r[mantle.nickel_contract_shapes.admission_boundary] The Nickel shape MUST stay
an authoring-time check. The Rust glue layer MUST remain the authority that
consumes the evaluated plan and enforces the real contract, and a Nickel
shape MUST NOT be reported as proof that a plan is admissible.

#### Scenario: A plan that satisfies the authoring shape

- GIVEN a plan that satisfies its Nickel shape
- WHEN the plan is converted for a build
- THEN the Rust layer MUST still validate the plan
- AND the Nickel result MUST NOT be reported as a build or release claim

### Requirement: The published standard-library surface is unchanged

r[mantle.nickel_contract_shapes.published_surface] The exported names of every
standard-library module, and the per-field documentation of every shape that
carries documentation, MUST survive the repair.

#### Scenario: A consumer importing a repaired module

- GIVEN a consumer that imports a repaired standard-library module
- WHEN the consumer reads the module's exported names
- THEN every previously exported name MUST still be present
- AND the documented fields MUST still be documented

### Requirement: The repair decision and its evidence are recorded

r[mantle.nickel_contract_shapes.decision_record] The repair MUST be recorded
as an architecture decision, the defect note MUST record the resolution, and
the retained gate transcript MUST be evidence for the recorded revision only.

#### Scenario: Reporting the repaired gate

- GIVEN a repaired revision with a retained first-party gate transcript
- WHEN the result is reported
- THEN the report MUST quote the recorded transcript
- AND it MUST NOT claim a bootstrap, release, reproducibility, or parity
  result
