## ADDED Requirements

### Requirement: Compiled evaluation backends stay behind `crunch-eval`

The system MUST treat any future Cranelift- or LLVM-based evaluator as an
implementation detail of `crunch-eval`, not as a new dependency of the build,
store, or sandbox layers.

Downstream crates MUST continue to consume evaluated derivation-shaped data and
existing diagnostics/results boundaries rather than compiler-specific IR or
machine-code concerns.

#### Scenario: Current interpreter path stays the only shipped path

- GIVEN the current tree without a shipped compiled evaluator backend
- WHEN a user runs `crunch eval` or `crunch build`
- THEN evaluation flows through the existing interpreter/export path
- AND `crunch-glue`, `crunch-build`, and `crunch-store` do not require
  Cranelift or LLVM

#### Scenario: Future backend swap does not perturb downstream crates

- GIVEN a future experimental compiled evaluator backend behind `crunch-eval`
- WHEN the backend is selected for evaluation
- THEN downstream crates still receive the same derivation-shaped outputs and
  error boundary
- AND build scheduling, sandbox execution, substitution, and store persistence
  stay outside the compiler backend

### Requirement: Compiled evaluation work is profiling-gated future work

The project MUST treat compiled evaluation backends as future optimization work
that starts only after measurement shows Nickel evaluation is a meaningful
bottleneck on real crunch workloads.

Repo docs and main specs MUST keep the interpreter/export path labeled as the
shipped runtime until a compiled backend exists in the runtime path.

#### Scenario: No evidence means no backend commitment

- GIVEN no benchmark or profiling evidence that evaluation dominates runtime
- WHEN planning architecture work
- THEN Cranelift and LLVM remain optional future work
- AND crunch does not add them as current required dependencies

#### Scenario: Evidence opens the door to an experiment

- GIVEN benchmark or profiling evidence that Nickel evaluation is a material
  bottleneck for package sets, self-build, or project-resolution flows
- WHEN future work is prioritized
- THEN the project may start a compiled evaluator experiment
- AND the experiment still stays behind the `crunch-eval` boundary

### Requirement: First native-code evaluator experiment prefers Cranelift

The first native-code evaluator experiment MUST prefer Cranelift by default.

LLVM MAY be considered later, but it MUST stay optional unless a shipped
backend proves that LLVM-specific optimization depth, ahead-of-time tooling, or
external toolchain integration is necessary.

#### Scenario: Early experiment chooses lighter backend

- GIVEN the project starts its first compiled evaluator prototype
- WHEN choosing a backend family
- THEN Cranelift is the default first choice
- AND the choice is justified as a lighter, faster-to-iterate option for
  runtime evaluation work

#### Scenario: LLVM remains a justified later option

- GIVEN a later phase where measured goals cannot be met with the first
  backend choice
- WHEN reassessing backend options
- THEN LLVM may be adopted for the compiled evaluator path
- AND the project records the LLVM-specific reason before making it required

### Requirement: Compiled backends preserve Nickel evaluation semantics

Any compiled evaluator backend MUST preserve the current Nickel evaluation
semantics for the supported subset, including contract failures, merge
resolution, recursive-record behavior, import resolution, and derivation
extraction shape.

A compiled backend MUST be treated as an optimization path, not as permission
to fork the language semantics.

#### Scenario: Contract failure matches interpreter behavior

- GIVEN a derivation fixture that fails under Nickel contract validation today
- WHEN it is evaluated through a future compiled backend for a supported subset
- THEN the backend reports failure rather than accepting invalid data
- AND the failure stays aligned with interpreter-defined semantics

#### Scenario: Derivation extraction stays shape-compatible

- GIVEN a package-set or nested-derivation fixture accepted by the current
  interpreter path
- WHEN a future compiled backend evaluates the same supported input
- THEN the extracted derivation data has the same logical shape
- AND downstream conversion/build code does not require backend-specific cases
