# Specification: Distributed C and C++ unit plans

## ADDED Requirements

### Requirement: C and C++ distribution uses an explicit frontend graph

r[distributed_compilation.cc_explicit_graph] Mantle MUST provide an opt-in
typed Nickel C/C++ graph frontend that lowers generators, individual
translation-unit compiles, ordered archives/links and final package outputs to
ordinary native dynamic plans. Inputs, dependencies, named outputs and
store-path tools MUST be explicit. Pure validation/lowering MUST stay in a
frontend core; the generic scheduler/store MUST NOT acquire compiler-specific
graph logic, command-log inference or remote frontend evaluation.

#### Scenario: Independent objects form a linked application

- GIVEN a valid graph with two independent compiles, a library archive and a final link
- WHEN the producer emits and the generic Worker admits its plan
- THEN the objects MUST be eligible for concurrent pool execution
- AND archive/link actions MUST have explicit finished-input edges and a bound final package root

#### Scenario: A declared generated dependency is unbound

- GIVEN a graph declares a generated-header input reference without a matching generator output edge
- WHEN frontend validation/lowering runs
- THEN Mantle MUST reject the unbound reference with a typed blocker
- AND it MUST NOT infer an edge from a local scratch file or a previous command log

### Requirement: Full header-search and tool inputs determine execution

r[distributed_compilation.cc_complete_inputs] A compile action MUST bind its
source content, complete ordered include-search trees and quote-relative
layout, forced includes, generated outputs, declared environment and full
compiler/resource/sysroot/runtime/helper inputs. Adding a previously absent
header in an admitted search root MUST affect identity. A previously learned
depfile or cache key MUST NOT substitute for the current complete input
closure. Undeclared roots, escaping paths and ambient tool/header discovery
MUST fail before upload or execution.

#### Scenario: A new earlier header shadows a previous match

- GIVEN an unchanged source and a header previously resolved from a later search root
- WHEN a matching header is added to an earlier declared root
- THEN the input/action identity MUST change and compilation MUST use the new admitted search result
- AND the old object MUST NOT satisfy the new action

#### Scenario: Host include directory is available only on one machine

- GIVEN a compile references an undeclared absolute include or environment search path
- WHEN remote frontend admission runs
- THEN Mantle MUST reject it rather than upload or mount the host directory
- AND worker-installed headers MUST NOT silently repair the missing input

### Requirement: Supported compiler modes are admitted explicitly

r[distributed_compilation.cc_supported_fragment] The distributed frontend
MUST admit only its declared pinned C/C++ tool/target/flag fragment, using the
existing bounded compile classifier as a necessary shape gate. Unknown flags,
response files, plugins, modules/PCH, undeclared side outputs, coverage/profile,
save-temps, LTO and unbound host/time-derived behavior MUST fail closed unless
separately admitted by a complete contract. Archive/link actions and requested
depfiles MUST have explicit ordered inputs and declared outputs. The ordinary
Make/local-driver route MUST retain its separate behavior; explicit unit-plan
failure MUST NOT turn into opaque remote forwarding.

#### Scenario: Declared C++ toolchain is sufficient

- GIVEN a supported compile-only C++ action with declared compiler, target sysroot and matching runtime
- WHEN it is classified and lowered with explicit archive/link actions
- THEN all semantic flags and tool identities MUST remain bound
- AND final linking MUST require the declared C++ runtime rather than worker PATH discovery

#### Scenario: Unknown side-effectful option is supplied

- GIVEN a distributed unit requests an unsupported option, response file or undeclared compiler output
- WHEN frontend admission runs
- THEN it MUST report an unsupported-fragment blocker before dispatch
- AND it MUST NOT forward the command remotely or claim that fragment is supported

### Requirement: Objects use ordinary verified native goal execution

r[distributed_compilation.cc_artifact_execution] Every lowered C/C++ action
MUST execute through the generic native goal/pool path and existing fenced,
full-key/content-verified result admission. Archives, links and install
consumers MUST wait for complete verified current outputs. Semantic input,
flag, generator, target or tool changes MUST invalidate affected actions;
placement changes MUST NOT. The local compile-cache/slot daemon, a broker
socket, a prior object or successful transport MUST NOT confer execution or
output trust.

#### Scenario: Objects move between workers

- GIVEN identical concrete inputs and logical prefix for an admitted compile action
- WHEN another eligible worker executes it and the link runs on a third location
- THEN the compile action identity MUST remain unchanged
- AND the link MUST consume the complete verified object/library closure through ordinary inputs

#### Scenario: A current compile fails after a previous successful build

- GIVEN an old accepted object and a changed source rejected by the real compiler
- WHEN the current compile goal fails
- THEN archive/link dependents MUST NOT use the old object as current success
- AND no successful final result for the changed graph MUST be published

### Requirement: Acceptance exercises real C and C++ consumers

r[distributed_compilation.cc_acceptance] Acceptance MUST preserve and execute
the existing `greet` workload's program/library/header semantics plus a real
C++ generated-header consumer. It MUST retain local/one-/two-worker builds,
actual compiler overlap, same-plan artifact comparison, independent expected
runtime/install outcomes, input/tool perturbations, rejection controls and a
fresh-store repeat. Complete include-tree invalidation MAY be conservative;
minimal recompilation MUST NOT be claimed without evidence. Loopback and
structural gates MUST NOT establish independent hosts, arbitrary build-system
support, StageX/GCC bootstrap acceptance, compiler correctness or release
eligibility.

#### Scenario: Distributed outputs are used by a real consumer

- GIVEN the admitted C/C++ fixtures and pinned tools
- WHEN pooled compilation, archive/link and installation succeed
- THEN the installed executables and a linked library consumer MUST run with independently expected results
- AND retained identities/receipts MUST show actual verified compile work rather than cached file presence alone

#### Scenario: A generated header changes

- GIVEN a successful graph whose consumer uses a generator's header
- WHEN only the declared generator output changes
- THEN its dependent compile and final application MUST use the new verified output
- AND the executed result MUST reflect the independently expected new value
