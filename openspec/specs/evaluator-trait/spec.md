# evaluator-trait Specification

## Purpose

This spec defines the dedicated-thread Nickel evaluator boundary and handle
protocol used by the system-config pipeline.
## Requirements
### Requirement: TRAIT-1 On-thread NickelEvaluator interface

`crunch-system` MUST define an object-safe `NickelEvaluator` trait for the
on-thread Nickel runtime boundary.
ID: systemconfig.evaluator.trait.trait1

The trait MUST include `evaluate_file`, `merge`, `call`, `get_field`,
`is_function`, and `to_json` methods. Each method operates on on-thread
`NickelValue` references and returns either `NickelValue`, `serde_json::Value`,
or `EvalError`. `crunch-eval` MUST remain independent of `crunch-system`; the
production evaluator adapter is constructed in `crunch-system` by wrapping
`crunch-eval` APIs.

#### Scenario: Loader validation uses the evaluator trait on the eval thread
ID: systemconfig.evaluator.trait.trait1.scenario

- GIVEN a loaded module value on the evaluator thread
- WHEN the loader checks `interface` and `impl`
- THEN it calls `get_field` and `is_function` through `NickelEvaluator`
- AND no filesystem I/O is required for the validation step

### Requirement: TRAIT-2 NickelValue stays on the eval thread

`NickelValue` MUST be a type alias for `crunch_eval::Expr` and MUST remain an
on-thread value representation.
ID: systemconfig.evaluator.trait.trait2

`NickelValue` MUST implement `Clone` and MUST NOT be treated as `Send` or
`Sync`. Raw `NickelValue` values MUST stay on the dedicated evaluator thread.
Cross-thread communication MUST use `ValueId` handles or serialized data,
rather than moving `NickelValue` across the channel boundary.

#### Scenario: Async callers use handles instead of moving Expr values
ID: systemconfig.evaluator.trait.trait2.scenario

- GIVEN an async task driving the system-config pipeline
- WHEN it needs to refer to a Nickel value owned by the evaluator thread
- THEN it stores a `ValueId` handle
- AND it does not receive a raw `NickelValue` over the channel

### Requirement: TRAIT-3 EvalOptions resource contract

`EvalOptions` MUST provide per-call resource and import configuration for the
on-thread evaluator.
ID: systemconfig.evaluator.trait.trait3

At minimum, `EvalOptions` MUST include `timeout: Option<Duration>` and
`import_paths: Vec<PathBuf>`. Timeout defaults to 60 seconds when the caller
does not override it. Memory limits MAY be added later if the Nickel runtime
exposes a suitable API.

#### Scenario: Module impl uses caller-provided timeout
ID: systemconfig.evaluator.trait.trait3.scenario

- GIVEN a module impl invocation with `EvalOptions { timeout = 5s, .. }`
- WHEN the evaluator services the request
- THEN the timeout bound for that request is 5 seconds
- AND other calls may use different timeout values

### Requirement: TRAIT-4 Import sandboxing

The production evaluator implementation MUST restrict Nickel import resolution
to the paths named in `EvalOptions.import_paths`.
ID: systemconfig.evaluator.trait.trait4

Imports resolving outside those directories MUST fail with
`EvalError::ImportDenied`. The default allowlist depends on what is being
evaluated: module evaluation uses the module directory plus crunch's embedded
stdlib directory, while inventory evaluation uses the inventory file directory
plus crunch's embedded stdlib directory and MUST NOT implicitly add the module
directory.

#### Scenario: Import outside the allowlist is rejected
ID: systemconfig.evaluator.trait.trait4.scenario

- GIVEN a module that imports a file outside the module directory and stdlib
- WHEN the evaluator resolves imports for that module
- THEN the request fails with `EvalError::ImportDenied`
- AND the diagnostic names the offending path and the allowed roots

### Requirement: TRAIT-5 EvalError type

`EvalError` MUST provide structured error variants for the evaluator boundary.
ID: systemconfig.evaluator.trait.trait5

At minimum it MUST include `NickelError(String)`, `Timeout`, and
`ImportDenied`. The type MUST implement `std::error::Error`, `Send`, and
`Sync` so that diagnostics can cross from the evaluator thread back to async
callers.

#### Scenario: Timeout error is serializable back to the caller
ID: systemconfig.evaluator.trait.trait5.scenario

- GIVEN a module impl that exceeds its timeout
- WHEN the evaluator request finishes
- THEN the caller receives `EvalError::Timeout`
- AND the error value is safe to move across the response channel

### Requirement: TRAIT-6 Pipeline wiring split

The pipeline MUST separate on-thread Nickel evaluation from async orchestration.
ID: systemconfig.evaluator.trait.trait6

Filesystem discovery and CLI orchestration happen on the async side. The async
side calls `EvalThreadHandle::evaluate_file()` to obtain `ValueId` handles for
module files, then continues through handle-based methods such as `get_field`,
`is_function`, `merge`, `call`, `to_json`, and `drop_value`. On the evaluator
thread, the production `NickelEvaluator` implementation operates on
`NickelValue` values referenced by those handles. This split MUST preserve the
functional-core boundary for validation and graph logic while keeping raw
Nickel values on the eval thread.

#### Scenario: File discovery feeds handle-based validation
ID: systemconfig.evaluator.trait.trait6.scenario

- GIVEN a discovered module file path
- WHEN the async pipeline asks the evaluator thread to load it
- THEN `EvalThreadHandle::evaluate_file()` returns a `ValueId`
- AND later validation requests reference that handle rather than a raw Expr

### Requirement: TRAIT-7 Dedicated thread and handle protocol

The entire Nickel evaluation pipeline MUST run on a single dedicated OS thread
that owns the `NickelValue` table.
ID: systemconfig.evaluator.trait.trait7

The async side communicates with that thread through a typed request/response
protocol carrying `ValueId` handles, serialized JSON values, and string-backed
error details. `EvalThreadHandle` MUST expose the full handle-based surface
needed by loader and evaluator orchestration: `evaluate_file`, `get_field`,
`is_function`, `merge`, `call`, `to_json`, `drop_value`, and shutdown. Those
methods MUST accept or return `ValueId` handles instead of raw `NickelValue`
values wherever a request crosses the thread boundary. `ValidatedModule`
values that live on the async side MUST hold `ValueId` handles rather than raw
`NickelValue` references. The evaluator thread is created once per
`crunch system` invocation and is shut down after all module work completes.

#### Scenario: ValidatedModule stores handles, not raw Nickel values
ID: systemconfig.evaluator.trait.trait7.scenario

- GIVEN a module that passed structural validation
- WHEN the async pipeline stores the validated module metadata
- THEN the stored representation contains `ValueId` handles for `interface` and
  `impl`
- AND dropping the module later sends handle-based cleanup back to the eval
  thread

