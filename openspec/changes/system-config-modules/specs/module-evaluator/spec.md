## ADDED Requirements

### Requirement: EVAL-1 Topological sort

The evaluator MUST sort modules by their declared `inputs` dependencies before
module execution begins.
ID: systemconfig.module.evaluator.eval1

Modules with no inputs are evaluated first. The sort MUST be stable with
priority as the first tiebreak and module name as the second tiebreak.

#### Scenario: Priority and name stabilize modules with no dependencies
ID: systemconfig.module.evaluator.eval1.scenario

- GIVEN two modules with no `inputs`
- AND one has a lower priority number than the other
- WHEN the evaluator computes execution order
- THEN the lower-priority-number module is evaluated first
- AND equal-priority modules fall back to alphabetical order

### Requirement: EVAL-2 Cycle detection

The evaluator MUST detect circular dependencies before module evaluation begins.
ID: systemconfig.module.evaluator.eval2

The reported cycle MUST name all modules involved. Cycles formed through any
combination of explicit `inputs` edges and provider edges are invalid.

#### Scenario: Combined input and provider edges form a cycle
ID: systemconfig.module.evaluator.eval2.scenario

- GIVEN a dependency graph whose remaining nodes form a cycle
- WHEN topological sorting runs
- THEN the evaluator fails before module execution starts
- AND the diagnostic names every module participating in the cycle

### Requirement: EVAL-3 Settings validation

For each module instance, the evaluator MUST merge user-provided settings with
the module interface defaults by using Nickel contract merge.
ID: systemconfig.module.evaluator.eval3

Contract violations MUST surface as Nickel blame diagnostics that name the
module, the role, and the field path when that information is available.

#### Scenario: Invalid settings surface a blame diagnostic
ID: systemconfig.module.evaluator.eval3.scenario

- GIVEN an instance whose settings violate the module contract
- WHEN the evaluator merges interface defaults with the instance settings
- THEN the instance fails with a Nickel blame diagnostic
- AND the diagnostic names the module and role involved

### Requirement: EVAL-4 Impl invocation

The evaluator MUST call each module's `impl` function with a record containing
validated `settings`, `machine_name`, `role_name`, `upstream`, and `providers`
fields.
ID: systemconfig.module.evaluator.eval4

The return value MUST be a record. The evaluator MUST serialize that record to
JSON before handing it to the fragment collector.

#### Scenario: Impl receives the threaded context record
ID: systemconfig.module.evaluator.eval4.scenario

- GIVEN a validated module instance ready for execution
- WHEN the evaluator calls the module's `impl`
- THEN the call arguments include settings, machine name, role name, upstream,
  and providers
- AND the successful result is serialized to JSON for later stages

### Requirement: EVAL-5 Export threading

When a module impl returns `output.exports`, the evaluator MUST make those
exports available to downstream modules that declare the module as an explicit
input.
ID: systemconfig.module.evaluator.eval5

Downstream modules MUST see the exports under `upstream.<module_name>`. If an
explicit input names a module that was excluded by loader validation, the
downstream module MUST fail with a chained diagnostic referencing the
originating loader failure instead of proceeding with missing upstream data.

#### Scenario: Downstream module reads upstream exports
ID: systemconfig.module.evaluator.eval5.scenario

- GIVEN module `nginx` declares `inputs = ["sshd"]`
- AND `sshd` returns `output.exports`
- WHEN `nginx` is evaluated
- THEN its `upstream.sshd` input contains the exported data from `sshd`
- AND if `sshd` had been excluded by loader validation, `nginx` would instead
  fail with a chained diagnostic referencing that loader failure

### Requirement: EVAL-6 Provider threading

The evaluator MUST, after applying any machine filter from CLI-4, use only the
remaining selected-machine instances when it constructs provider edges,
collects produced provider outputs by type, and exposes each consumed provider
type as an ordered array under `providers.<type>`.
ID: systemconfig.module.evaluator.eval6

Provider edges participate in the same topological ordering as explicit
`inputs`. When multiple modules produce the same provider type, the consuming
module receives an array ordered by evaluation order. If a module consumes a
provider type that no loaded module produces, the evaluator MUST record a
`SystemConfigWarning`, MUST supply an empty array for that provider type, and
MUST continue evaluation. If the only would-be producers for a consumed
provider type were excluded by loader validation, the consumer MUST instead
fail with a chained diagnostic referencing the originating loader failure
rather than being treated as a normal orphan-provider warning.

#### Scenario: Orphan provider consumption becomes a warning
ID: systemconfig.module.evaluator.eval6.scenario

- GIVEN a module that consumes provider type `firewall`
- AND no loaded module produces provider type `firewall`
- WHEN the evaluator prepares provider inputs
- THEN the module receives `providers.firewall = []`
- AND `SystemPipelineResult.warnings` records the orphan-provider warning
- AND if the only `firewall` producer had been excluded by loader validation,
  the consumer would instead fail with a chained diagnostic

### Requirement: EVAL-7 Determinism

Given the same modules, inventory, and overrides, the evaluator MUST produce
identical output fragments.
ID: systemconfig.module.evaluator.eval7

The evaluator MUST NOT depend on randomness, timestamps, or ambient mutable
state.

#### Scenario: Repeated runs produce identical fragment output
ID: systemconfig.module.evaluator.eval7.scenario

- GIVEN the same module set and inventory are evaluated twice
- WHEN both runs complete successfully
- THEN the two fragment collections are byte-for-byte equivalent after JSON
  serialization

### Requirement: EVAL-8 Fixed limits

The evaluator MUST enforce configurable limits before execution begins.
ID: systemconfig.module.evaluator.eval8

The required limits are maximum dependency-chain depth 256, maximum provider
types per module 32, and maximum total provider edges 4096.

#### Scenario: Provider-edge limit stops evaluation early
ID: systemconfig.module.evaluator.eval8.scenario

- GIVEN a module set whose provider graph exceeds the total provider-edge limit
- WHEN the evaluator validates the graph
- THEN evaluation fails before any module impl is called
- AND the diagnostic names the exceeded limit

### Requirement: EVAL-9 Error collection

Evaluation failures in one module MUST NOT prevent evaluation of independent
modules.
ID: systemconfig.module.evaluator.eval9

The evaluator MUST collect module failures in `SystemPipelineResult.errors`,
collect non-fatal warnings in `SystemPipelineResult.warnings`, and continue
processing modules that do not depend on the failed module. Modules that depend
on a failed module MUST fail with chained diagnostics. At the end of the
pipeline, continued module-level progress MUST still collapse into the per-
machine terminal outcomes required by ERR-4: a selected machine that cannot
produce a complete phase payload becomes `MachineOutcome::Failed`, while an
independent selected machine may still reach a successful terminal outcome.

#### Scenario: Independent modules continue after one failure
ID: systemconfig.module.evaluator.eval9.scenario

- GIVEN module `a` on machine `server1` fails during evaluation
- AND module `b` on independent machine `server2` is independent of `a`
- WHEN the evaluator continues execution
- THEN `b` is still evaluated
- AND the final result records `MachineOutcome::Failed` for `server1`
- AND the final result records a successful terminal outcome for `server2`
- AND the failure diagnostic for `a` remains in `SystemPipelineResult.errors`

### Requirement: EVAL-10 Nickel evaluation resource bounds

Nickel evaluation of each module MUST be bounded by a configurable wallclock
timeout whose default value is 60 seconds per module `impl` call.
ID: systemconfig.module.evaluator.eval10

When a module exceeds the timeout, its evaluation MUST fail with a diagnostic
naming the module and elapsed time, while independent modules continue per
EVAL-9. Memory bounding MAY be added later if the Nickel runtime supports it.

#### Scenario: Timed-out module does not block independent modules
ID: systemconfig.module.evaluator.eval10.scenario

- GIVEN a module impl that does not finish before its timeout
- WHEN the timeout expires
- THEN that module fails with a timeout diagnostic
- AND independent modules continue evaluating

### Requirement: EVAL-11 Injected evaluator boundary

The async evaluator orchestrator MUST receive Nickel evaluation capability
through the handle-based boundary defined by TRAIT-6 and TRAIT-7, while the
on-thread execution engine behind that handle MUST implement `NickelEvaluator`
as defined by TRAIT-1 through TRAIT-5.
ID: systemconfig.module.evaluator.eval11

Tests for async orchestration MAY substitute a fake handle-compatible boundary
that returns deterministic `ValueId`/JSON results without a real Nickel
runtime. Tests for the on-thread evaluator engine MAY substitute a mock
`NickelEvaluator`. The boundary choice MUST remain consistent with loader and
evaluator code that operate on `ValueId` plus handle methods rather than raw
`NickelValue` values.

#### Scenario: Tests replace the Nickel runtime with a mock evaluator
ID: systemconfig.module.evaluator.eval11.scenario

- GIVEN a unit test for dependency ordering or export threading on the async
  orchestrator path
- WHEN the evaluator is constructed for the test
- THEN the test may inject a fake handle-compatible boundary backed by
  deterministic `ValueId`/JSON behavior
- AND no real Nickel runtime is required for that test

### Requirement: EVAL-12 Cross-reference validation

Before calling any module `impl`, the evaluator MUST validate inventory-module
cross-references.
ID: systemconfig.module.evaluator.eval12

The evaluator MUST reject inventory service keys that do not match a loaded
module identity, instance roles not declared in the target module's
`interface.roles`, and instance machines not declared in the inventory. After
applying any machine filter from CLI-4, cross-reference validation MUST run
only for the remaining selected-machine instances so unselected instances do
not produce diagnostics. If a service names a module that the loader excluded
after structural validation, the evaluator MUST report that case as a chained
diagnostic referencing the originating loader failure, not as an
indistinguishable bare missing-module error.

#### Scenario: Unknown role is rejected before impl execution
ID: systemconfig.module.evaluator.eval12.scenario

- GIVEN an inventory instance naming a role that the module does not declare
- WHEN cross-reference validation runs
- THEN the evaluator records a cross-reference error
- AND it does not call the module's `impl` for that instance
