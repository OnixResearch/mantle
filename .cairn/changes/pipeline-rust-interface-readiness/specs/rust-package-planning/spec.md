# Specification: Rust interface readiness pipelining

## ADDED Requirements

### Requirement: Experimental compiler capability is explicitly admitted

r[rust_package_planning.phase_readiness.compiler_gate] Before experimental
execution, Mantle MUST admit exact compiler/Cargo source and binary identities,
reviewed patch identities, sysroot/runtime closure, semantic options, policy,
and supported phase protocol. The Headstart incremental early/full metadata SVH
defect MUST be resolved by a source fix demonstrated under normal metadata
hashing, including unchanged warm and edit/revert cases. Unsupported or
unresolved capability MUST block experimental execution before side effects.
Neither nightly naming nor a flag accepted syntactically establishes support.

#### Scenario: Fixed compatible provider is selected

- GIVEN a declared provider with a source-fixed incremental identity regression and complete admitted runtime inputs
- WHEN the operator explicitly selects experimental interface readiness
- THEN the executed flags/config and exact provider/protocol identities MUST be bound to the execution identity
- AND default Cargo/native modes MUST remain unchanged when no experiment is selected

#### Scenario: Known bug is bypassed instead of fixed

- GIVEN an unfixed provider, or evidence relying only on disabled incremental compilation, cache flushing, or alternate metadata hashing
- WHEN experimental execution is requested
- THEN the compiler gate MUST remain blocked
- AND Mantle MUST NOT claim the incremental-identity prerequisite passed

#### Scenario: Toolchain or protocol does not match

- GIVEN a missing capability, mismatched compiler/sysroot identity, or unsupported notification protocol
- WHEN the session is admitted
- THEN Mantle MUST report the exact unsupported or mismatched input
- AND MUST NOT enable a hidden environment fallback or silently run another mode as the experiment

### Requirement: Initial experiment retains one-sandbox output admission

r[rust_package_planning.phase_readiness.sandbox_trial] The initial qualification
trial MUST execute the declared patched Cargo/rustc pair inside one existing
offline Cargo sandbox with explicit opt-in policy and complete admitted inputs.
It MUST preserve offline/locked source handling, required compiler policy,
strict provisional-session resource controls, and ordinary whole-derivation
publication after success. The trial MUST NOT require mutable live files from
another sandbox or remote producer.

#### Scenario: Same declared toolchain is compared off and on

- GIVEN admitted equal source/lock/vendor/toolchain/runtime inputs and isolated comparable cache cohorts
- WHEN the trial alternates ordinary and experimental modes
- THEN it MUST retain actual command selection, diagnostics, statuses, resource observations, and final output observations for both modes
- AND success MUST still pass existing whole-package output admission

#### Scenario: Upstream resume cannot satisfy the selected bounds

- GIVEN a patched Cargo implementation that resumes processes without enforcing the selected strict session reservation policy
- WHEN the trial evaluates its capabilities
- THEN execution MUST remain blocked unless an admitted adapter/provider actually enforces those bounds
- AND post-resume accounting or RSS sampling alone MUST NOT be called enforcement

#### Scenario: An input or declared compiler policy is missing

- GIVEN incomplete source/runtime inputs or a direct-path selection that would bypass required compiler policy
- WHEN trial execution is requested
- THEN Mantle MUST reject or block before dispatch rather than read ambient caches, libraries, network sources, or credentials

### Requirement: Readiness requirements preserve analysis and execution barriers

r[rust_package_planning.phase_readiness.typed_requirements] The Rust planning
core MUST distinguish exact-producer-attempt interface-ready,
full-metadata-ready, and link-artifact-ready requirements from final successful
confirmation. Analysis MAY use admitted early metadata for supported rlib
inputs. Code generation MUST require matching full metadata. Linking MUST
require completed libraries, and cross-crate LTO MUST require them before code
generation. Proc-macro/dylib consumption and build-script execution MUST wait
for the required completed executable artifacts. The core MUST remain pure.

#### Scenario: Analysis begins before a producer finishes checking bodies

- GIVEN a supported producer emits admitted interface metadata while its body checking remains active
- WHEN all of a dependent's analysis requirements are satisfied and resources permit
- THEN the dependent analysis MAY begin before producer completion
- AND its results MUST remain provisional until the producer-attempt closure is confirmed

#### Scenario: Generic code generation or LTO needs final producer data

- GIVEN dependent analysis used early metadata and needs a producer's generic/inline MIR or cross-crate LTO library
- WHEN code generation becomes eligible
- THEN matching full metadata and, for cross-crate LTO, completed libraries MUST be present and admitted first
- AND absent or identity-mismatched producer data MUST fail the attempt rather than use a stale file

#### Scenario: Proc macro or generated build-script output is needed

- GIVEN a consumer requires a proc-macro/dylib or an executed build script's generated output
- WHEN only early interface metadata exists
- THEN consumption or script execution MUST remain ineligible
- AND the compiler's ability to analyze a script early MUST NOT authorize running its provisional executable

### Requirement: Native phase execution uses one bounded local session

r[rust_package_planning.phase_readiness.execution] Mantle MUST extend the
existing Rust core/application execution boundary with bounded live phase
observations and deterministic ready-unit ordering, using the provisional
session's exact attempt identity, confirmation, reservation, failure, and
teardown contracts. Each admitted unit MUST use one actual compiler invocation
for its supported metadata/code emission. Full source/compiler/options/policy
identity MUST remain in existing action admission. Generic scheduler Done,
ordinary dynamic-plan UnitOutput, and signed store-result semantics MUST NOT
be weakened by an early file or notification.

#### Scenario: Supported native session overlaps compiler stages

- GIVEN fixed admitted compiler capability and a supported topology with ordinary and early metadata modes
- WHEN the native application runs a phase-aware session
- THEN actual supported notifications MUST drive the declared Rust readiness requirements
- AND the observation trace MUST show stage overlap without repeating analysis in separate synthetic compiler actions

#### Scenario: Producer crashes or notification belongs to another attempt

- GIVEN a dependent has begun analysis and the producer crashes, omits final output, or emits a stale/mismatched notification
- WHEN the failure is observed
- THEN affected provisional work MUST be invalidated and owned descendants torn down under the session budget
- AND no final cache result, manifest, signed PathInfo, or completed generic goal MUST be created from the affected early metadata

#### Scenario: Wrapper buffers control until compiler exit

- GIVEN an otherwise selected wrapper cannot provide bounded live phase forwarding
- WHEN the session evaluates the integration
- THEN it MUST reject that integration or use a separately admitted policy-compatible live path
- AND MUST NOT infer early readiness by parsing delayed human log output

### Requirement: Acceptance is differential, resource-bounded, and scoped

r[rust_package_planning.phase_readiness.acceptance] Experimental acceptance
MUST use retained same-toolchain serial/ordinary-full-metadata/early-interface
comparisons on declared supported workload, target, profile, feature, and
optimization cells. It MUST exercise successful output behavior, failure and
crash propagation, clean/warm/incremental identity, low-slot liveness, resource
limits, and teardown using the prerequisite evidence mechanisms. Required
blocked or unrun cells MUST NOT count as passes. Timing MUST remain diagnostic;
no general speedup, compiler correctness, full Cargo compatibility, self-hosting,
reproducibility, or release claim follows from the trial or native session.

#### Scenario: Both implementation stages have real evidence

- GIVEN a qualified one-sandbox trial and a supported native phase executor
- WHEN implementation acceptance is requested
- THEN both stages MUST have their own retained successful and negative runtime observations
- AND passing only the Cargo trial MUST NOT satisfy native executor requirements

#### Scenario: Wide workload has no measurable improvement

- GIVEN comparable retained runs whose elapsed ranges overlap or whose resource cost increases
- WHEN results are reported
- THEN Mantle MUST report those observations without asserting an unsupported speedup or silently excluding failed runs
- AND MUST preserve default behavior and the declared resource ceilings

#### Scenario: Full Mantle workspace exceeds the admitted unit-plan bound

- GIVEN a workload outside the declared source-slice or other compatibility limits
- WHEN the experimental report is produced
- THEN the workload MUST be classified as outside the admitted scope
- AND small-fixture success MUST NOT be relabelled as full-workspace, self-hosting, or bootstrap evidence
