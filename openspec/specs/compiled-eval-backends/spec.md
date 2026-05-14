# Compiled Evaluation Backends Specification

## Purpose

Defines constraints for optional compiled evaluator experiments, including the
`crunch-eval` boundary, profiling gates, feature-gated prototype scope,
benchmark guardrails, and future semantic-equivalence requirements.

## Requirements

### Requirement: Compiled evaluation backends stay behind `crunch-eval`

The system MUST treat any future Cranelift- or LLVM-based evaluator as an
r[compiled-eval.backend-boundary]
implementation detail of `crunch-eval`, not as a new dependency of the build,
store, or sandbox layers.

Downstream crates MUST continue to consume evaluated derivation-shaped data and
existing diagnostics/results boundaries rather than compiler-specific IR or
machine-code concerns.

#### Scenario: Current interpreter path stays the default shipped path
r[compiled-eval.backend-boundary.default]

- GIVEN the current tree without a default shipped compiled evaluator backend
- WHEN a user runs `mantle eval` or `mantle build`
- THEN evaluation flows through the existing interpreter/export path
- AND `crunch-glue`, `crunch-build`, and `crunch-store` do not require
  Cranelift or LLVM

#### Scenario: Future backend swap does not perturb downstream crates
r[compiled-eval.backend-boundary.swap]

- GIVEN a future experimental compiled evaluator backend behind `crunch-eval`
- WHEN the backend is selected for evaluation
- THEN downstream crates still receive the same derivation-shaped outputs and
  error boundary
- AND build scheduling, sandbox execution, substitution, and store persistence
  stay outside the compiler backend

### Requirement: Compiled evaluation work is profiling-gated future work

The project MUST treat compiled evaluation backends as future optimization work
r[compiled-eval.profiling-gate]
that starts only after measurement shows Nickel evaluation is a meaningful
bottleneck on real mantle workloads.

Repo docs and main specs MUST keep the interpreter/export path labeled as the
shipped runtime until a compiled backend exists in the runtime path.

#### Scenario: No evidence means no backend commitment
r[compiled-eval.profiling-gate.no-evidence]

- GIVEN no benchmark or profiling evidence that evaluation dominates runtime
- WHEN planning architecture work
- THEN Cranelift and LLVM remain optional future work
- AND mantle does not add them as current required dependencies

#### Scenario: Evidence opens the door to an experiment
r[compiled-eval.profiling-gate.evidence]

- GIVEN benchmark or profiling evidence that Nickel evaluation is a material
  bottleneck for package sets, self-build, or project-resolution flows
- WHEN future work is prioritized
- THEN the project may start a compiled evaluator experiment
- AND the experiment still stays behind the `crunch-eval` boundary

### Requirement: Benchmark gate and guardrails precede promotion

A compiled evaluator prototype MUST NOT be promoted beyond experiment status
r[compiled-eval.benchmark-guardrail]
unless checked-in benchmark evidence shows both an evaluation-dominant workload
and a material eval-phase win without unacceptable guardrail regressions.

The benchmark gate MUST use checked-in workloads that separate evaluation,
conversion, build-graph, substitution, and store phases where those phases are
observable. The initial promotion threshold is an eval-bearing workflow whose
`evaluation_wall_ns` exceeds 90% of `total_wall_ns`, plus a prototype that
improves the eval-focused workload by at least 2x while remaining within the
checked-in `benchmark_compare` thresholds for non-eval guardrails.

#### Scenario: Gate evidence is recorded before prototype promotion
r[compiled-eval.benchmark-guardrail.recorded]

- GIVEN a feature-gated compiled evaluator prototype exists
- WHEN maintainers consider making it broader than an experiment
- THEN they first record a same-host benchmark suite bundle and
  `benchmark_compare` transcript
- AND the transcript must show whether non-eval guardrails stayed within the
  documented thresholds

#### Scenario: Guardrail regression keeps prototype non-shipping
r[compiled-eval.benchmark-guardrail.regression]

- GIVEN the `benchmark_compare` transcript shows a non-eval guardrail outside
  the documented threshold
- WHEN the compiled evaluator workstream is closed
- THEN the prototype remains feature-gated and non-shipping
- AND the closeout records that further optimization work is tabled

### Requirement: First native-code evaluator experiment prefers Cranelift

The first native-code evaluator experiment MUST prefer Cranelift by default.
r[compiled-eval.cranelift-first]

LLVM MAY be considered later, but it MUST stay optional unless a shipped
backend proves that LLVM-specific optimization depth, ahead-of-time tooling, or
external toolchain integration is necessary.

#### Scenario: Early experiment chooses lighter backend
r[compiled-eval.cranelift-first.initial]

- GIVEN the project starts its first compiled evaluator prototype
- WHEN choosing a backend family
- THEN Cranelift is the default first choice
- AND the choice is justified as a lighter, faster-to-iterate option for
  runtime evaluation work

#### Scenario: LLVM remains a justified later option
r[compiled-eval.cranelift-first.llvm-later]

- GIVEN a later phase where measured goals cannot be met with the first
  backend choice
- WHEN reassessing backend options
- THEN LLVM may be adopted for the compiled evaluator path
- AND the project records the LLVM-specific reason before making it required

### Requirement: Backend seam remains private and API-preserving

A compiled-eval experiment MUST keep the existing public `crunch-eval` helpers
r[compiled-eval.private-backend-seam]
source-compatible for downstream callers. Any backend selector, request carrier,
or internal trait introduced for experiments MUST remain private until a later
OpenSpec change defines a public backend API.

#### Scenario: Existing callers remain source-compatible
r[compiled-eval.private-backend-seam.callers]

- GIVEN downstream crates call the existing `crunch-eval` evaluation helpers
- WHEN the private backend seam is added
- THEN those call sites do not need compiler-backend-specific arguments or types
- AND default evaluation still uses the Nickel interpreter backend

#### Scenario: Experimental backend types do not leak
r[compiled-eval.private-backend-seam.no-leak]

- GIVEN the internal backend uses request or trait types
- WHEN another crate depends on `crunch-eval`
- THEN it cannot depend on compiler-specific IR, JIT handles, or backend request
  carrier types

### Requirement: Feature-gated Cranelift prototype is subset-only

The Cranelift prototype MUST stay behind the optional `cranelift-proto` feature
r[compiled-eval.cranelift-prototype-subset]
and support only the documented flat-derivation literal subset in this change.
Unsupported Nickel constructs MUST be rejected by the prototype or left to the
interpreter path; they MUST NOT be silently accepted with changed semantics.

The supported subset is limited to required `name` and `builder` fields plus
optional `system`, `addressing_mode`, `args`, and `outputs` fields for a flat
record literal. Broader contracts, merges, imports, package sets, recursive
records, and nested derivation inputs are future-backend requirements, not
support claims for this prototype.

#### Scenario: Default build excludes prototype dependencies
r[compiled-eval.cranelift-prototype-subset.default]

- GIVEN the workspace is built without `--features cranelift-proto`
- WHEN `crunch-eval` compiles
- THEN Cranelift dependencies are not required by the default runtime path
- AND public interpreter helpers remain available

#### Scenario: Unsupported shape is not treated as compiled success
r[compiled-eval.cranelift-prototype-subset.unsupported]

- GIVEN a Nickel input that is not the documented flat-derivation literal subset
- WHEN the Cranelift prototype is asked to evaluate it
- THEN the prototype rejects the unsupported field or construct or leaves it outside the supported subset
- AND no downstream build or store code receives a backend-specific shape

### Requirement: Future expanded compiled backends preserve Nickel semantics

Any future expanded compiled evaluator backend MUST preserve the current Nickel
r[compiled-eval.future-semantics]
evaluation semantics for its supported constructs, including contract failures,
merge resolution, recursive-record behavior, import resolution, and derivation
extraction shape.

A compiled backend MUST be treated as an optimization path, not as permission
to fork the language semantics.

#### Scenario: Contract failure matches interpreter behavior
r[compiled-eval.future-semantics.contracts]

- GIVEN a derivation fixture that fails under Nickel contract validation today
- WHEN it is evaluated through a future expanded compiled backend for a supported
  subset
- THEN the backend reports failure rather than accepting invalid data
- AND the failure stays aligned with interpreter-defined semantics

#### Scenario: Derivation extraction stays shape-compatible
r[compiled-eval.future-semantics.shape]

- GIVEN a package-set or nested-derivation fixture accepted by the current
  interpreter path
- WHEN a future expanded compiled backend evaluates the same supported input
- THEN the extracted derivation data has the same logical shape
- AND downstream conversion/build code does not require backend-specific cases
