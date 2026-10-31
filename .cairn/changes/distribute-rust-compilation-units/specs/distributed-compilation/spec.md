# Specification: Distributed Rust unit-plan compilation

## ADDED Requirements

### Requirement: Rust distribution uses the existing generic unit graph

r[distributed_compilation.rust_unit_graph] Mantle MUST distribute the existing
opt-in Cargo unit-plan lane through ordinary native dynamic-plan admission and
pool goal dispatch. Signed producer plan/source outputs and selected root
bindings MUST be admitted before children. Rust package/target semantics MUST
remain in the frontend; the generic Worker/store MUST NOT gain Rust branches,
remote frontend evaluation or a host Cargo compilation fallback.

#### Scenario: One Rust build uses two workers

- GIVEN a supported unit plan with independent library branches and an explicit pool
- WHEN verified producer outputs are admitted and both branches become ready
- THEN the generic Worker MUST be able to run real rustc work on distinct workers concurrently
- AND the selected application MUST finish only after its bound final unit succeeds

#### Scenario: Invalid producer plan is not executed

- GIVEN a producer returns an oversized, misbound or malformed plan/source output
- WHEN normal dynamic-plan/source admission runs
- THEN Mantle MUST reject it under existing bounds and identity checks
- AND no emitted unit or fallback Cargo compilation MUST execute

### Requirement: Compiler execution and artifact targets are distinct

r[distributed_compilation.rust_platform_roles] The frontend MUST bind compiler
execution system, artifact target/sysroot, helper/linker runtime, proc-macro ABI
and build-script execution/target facts separately. Placement MUST use the
executable's admitted execution requirements, not infer them from the artifact
target. Unsupported combinations and undeclared native roles MUST fail before
upload or compiler execution; this change MUST NOT imply new target coverage.

#### Scenario: Host tools produce target artifacts

- GIVEN an admitted compiler execution system, a distinct declared target, and host proc-macro/script tools
- WHEN the frontend lowers and pool placement admits those units
- THEN executable consumers MUST run only on workers satisfying their host/runtime requirements
- AND target compilation MUST consume the declared target facts/sysroot rather than worker ambient facts

#### Scenario: Matching target does not authorize a wrong compiler host

- GIVEN a worker advertises the requested artifact target but cannot run the selected compiler or proc macro
- WHEN placement evaluates the concrete unit
- THEN it MUST reject that worker before transfer/execution
- AND it MUST NOT substitute a worker-installed compiler or silently change the target

### Requirement: Only complete verified dependency artifacts cross workers

r[distributed_compilation.rust_artifact_barriers] Rust consumers MUST wait for
current finished and verified dependency outputs, including full library
reference manifests, proc-macro runtime closure, script executable, target
facts, generated source, `OUT_DIR`, typed flags and declared native/linker
inputs. Missing transitive inputs MUST fail closed. Provisional metadata,
readiness events and previous-attempt files MUST NOT satisfy these barriers.

#### Scenario: Generated source reaches a different worker

- GIVEN a script successfully generates a value and typed flags on worker A
- WHEN a dependent compile is placed on worker B
- THEN B MUST receive the verified current generated source/flags and complete declared closure before rustc starts
- AND the final executed application MUST reflect the generated value

#### Scenario: A failed script leaves old output behind

- GIVEN a previous successful script output and a current script attempt that fails
- WHEN a dependent requests its generated files or flags
- THEN the failed attempt MUST NOT satisfy the dependency
- AND no signed successful final application MUST be published from the old files

### Requirement: Placement preserves semantic unit identity and signed reuse

r[distributed_compilation.rust_identity_reuse] Pool/worker/attempt placement
facts MUST NOT change canonical unit/action identities, `-C metadata`, stable
source remapping or named outputs. Semantic toolchain, arguments, environment,
target, source and declared dependency changes MUST invalidate affected actions.
Only completed verified artifact reuse is admitted; cross-worker incremental,
warm mutable workspace and live compiler state MUST NOT be shared by this lane.

#### Scenario: Unchanged unit moves to another worker

- GIVEN the same plan, semantic inputs and logical store prefix with an admitted signed unit result
- WHEN a later build selects another eligible worker
- THEN the unit/action identity MUST remain unchanged and trusted reuse MAY avoid execution
- AND reuse MUST identify its verified artifact basis rather than imply new remote compilation

#### Scenario: Native provider changes with a stable nominal unit label

- GIVEN a script and its consumers depend on a declared native provider
- WHEN only that provider's admitted identity changes
- THEN affected canonical derivation/action identities and compiled results MUST be recomputed
- AND a previous signed result for the old identity MUST NOT satisfy the new action

### Requirement: Rust distribution has real bounded acceptance evidence

r[distributed_compilation.rust_acceptance] Acceptance MUST retain real local,
one-/two-worker same-plan builds, actual rustc overlap, executed final program
results, host-sensitive script/proc-macro behavior, perturbations, failure
controls and a fresh-store repeat. Same-plan stable artifacts MUST be compared;
comparison to the default Cargo lane MUST use declared runtime semantics, not
assume identical metadata/binaries. Existing plan bounds, support blockers and
lane non-claims MUST remain explicit. Loopback MUST NOT claim independent-host
operation, broader Cargo compatibility, compiler correctness, Cargo-free
execution, bootstrap/release correctness or a general speedup.

#### Scenario: Working application is observed

- GIVEN a supported branching graph with a real proc macro, generated source and declared native input
- WHEN local and pooled clean builds complete under the same admitted toolchain
- THEN the final programs MUST execute with the independently expected results
- AND retained evidence MUST bind source/tool/plan/output identities and actual compilation/admission outcomes

#### Scenario: Unsupported full graph is not counted as coverage

- GIVEN a workspace or target mode exceeds existing unit-plan support or limits
- WHEN distributed lowering is requested
- THEN it MUST remain explicitly blocked with the applicable typed reason
- AND no smaller fixture, skip, local Cargo fallback or lifecycle gate MUST be reported as acceptance of that graph
