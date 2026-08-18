# Build Correctness Specification

## Purpose

Makes source-root capability reporting honest and requires strict hermetic execution for Onix release evidence.

## ADDED Requirements

### Requirement: Advertised source-root capability is executable or explicitly unsupported
r[mantle.build_correctness.source_root_capability] Mantle MUST NOT advertise a source-root operation as executable when every invocation unconditionally returns unavailable. The operation MUST either execute a bounded receipt-producing implementation or be omitted from executable command discovery and reported as an explicit unsupported capability with a deterministic reason.

#### Scenario: Supported source-root operation executes
- GIVEN the host and explicit inputs satisfy the declared source-root capability contract
- WHEN the operator invokes the source-root operation
- THEN Mantle MUST execute the bounded operation and emit a receipt identifying inputs, policy, observed capability, outputs, and non-claims.

#### Scenario: Unsupported source-root capability is honest
- GIVEN the implementation or host cannot support source-root execution
- WHEN command discovery or capability reporting runs
- THEN Mantle MUST report the operation as unsupported before execution and MUST NOT expose a command path whose only outcome is an unconditional unavailable error.

### Requirement: Source-root logic preserves core and shell boundaries
r[mantle.build_correctness.source_root_capability.boundary] Mantle MUST keep source-root planning and capability decisions pure over supplied observations while filesystem discovery, host probing, source loading, process execution, and receipt writing remain in the shell.

#### Scenario: Pure planning is testable without host setup
- GIVEN source-root capability observations and a requested operation in memory
- WHEN planning runs
- THEN it MUST produce an execute or unsupported decision without reading files, environment state, clocks, network resources, or processes.

### Requirement: Onix release profiles require strict hermeticity
r[mantle.build_correctness.onix_release_strict_hermeticity] Mantle MUST require strict hermetic mode for Onix release evidence and MUST fail before pass evidence when declared sandbox, network, environment, path, clock, or tool restrictions cannot be enforced.

#### Scenario: Clean strict execution may pass
r[mantle.build_correctness.hermetic_handoff.fixtures.positive]
- GIVEN the executor enforces every strict restriction and records matching evidence
- WHEN the Onix release profile evaluates the build
- THEN the hermeticity contribution MAY pass.

#### Scenario: Host influence fails strict release evidence
r[mantle.build_correctness.hermetic_handoff.fixtures.negative]
- GIVEN execution observes undeclared host tools, ambient environment, network access, unstable paths, clock dependence, or unenforced sandbox policy
- WHEN the Onix release profile evaluates the build
- THEN the strict hermeticity contribution MUST fail.

#### Scenario: Practical mode cannot satisfy strict profile
- GIVEN a practical-mode build completes and emits diagnostic or development evidence
- WHEN an Onix release profile requires strict hermeticity
- THEN the practical receipt MUST NOT satisfy the strict evidence field or be promoted by summary metadata.

### Requirement: Build boundary remains bounded
r[mantle.build_correctness.hermetic_handoff.docs] Mantle documentation MUST distinguish executable source-root capability, explicit unsupported capability, strict release evidence, and practical diagnostic evidence.

#### Scenario: Strict evidence avoids overclaim
- GIVEN strict hermetic execution passes
- WHEN the result is documented
- THEN it MUST NOT claim compiler correctness, source correctness, semantic equivalence, universal reproducibility, deployment safety, or release eligibility outside the configured profile.
