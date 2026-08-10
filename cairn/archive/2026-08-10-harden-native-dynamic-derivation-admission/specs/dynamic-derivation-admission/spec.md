# Dynamic Derivation Admission

## ADDED Requirements

### Requirement: Staged pure admission core

r[dynamic_derivation_admission.staged_core] Mantle MUST admit native dynamic derivations through private parsed, validated, identity-resolved, and registry-ready states computed by a pure deterministic core over explicit bytes, policy, prefix, limits, and parent facts.

#### Scenario: Valid candidate reaches registry-ready state

- GIVEN bounded valid derivation bytes, supported semantic policy, one logical store prefix, and complete parent facts
- WHEN staged admission runs
- THEN each state transition MUST validate its declared invariants before constructing the next private type
- AND the final value MUST contain the complete identity and registration plan

#### Scenario: Shell effect enters the core

- GIVEN an implementation reads castore, inspects the registry, logs, mutates goals, reads a clock, accesses a file, or starts a process inside the admission core
- WHEN boundary validation runs
- THEN validation MUST fail with a deterministic core-purity finding
- AND the effect MUST remain in the Worker shell

### Requirement: Complete parent identity is mandatory

r[dynamic_derivation_admission.complete_parent_identity] Native dynamic derivation identity MUST use one explicit admitted hash fact for every direct parent derivation. Missing, duplicate, conflicting, or wrong-prefix parent facts MUST fail before identity calculation, and Mantle MUST NOT substitute an all-zero or other sentinel digest.

#### Scenario: Every parent fact is present

- GIVEN each referenced direct parent has one admitted hash fact under the selected logical prefix
- WHEN identity resolution runs
- THEN Mantle MUST calculate the native BLAKE3 derivation identity from those exact facts
- AND the resulting configured-prefix path MAY proceed to registry planning

#### Scenario: Parent fact is unavailable

- GIVEN a referenced parent has no admitted hash fact or has conflicting or wrong-prefix facts
- WHEN identity resolution runs
- THEN Mantle MUST return a stable parent-identity blocker
- AND it MUST NOT calculate a fallback identity or emit a registry-ready value

### Requirement: Bounded traditional and versioned forms

r[dynamic_derivation_admission.versioned_forms] Mantle MUST recognize only declared traditional and versioned dynamic derivation forms under named byte, collection, and recursive-depth bounds. Unknown versions, excessive depth, empty dynamic requests, and unsupported output semantics MUST fail before identity resolution.

#### Scenario: Supported versioned input tree is admitted

- GIVEN a supported versioned derivation contains a recursive dynamic-input tree within every named bound
- WHEN parsing and semantic validation run
- THEN Mantle MUST preserve the requested direct and dynamic outputs in deterministic traversal order
- AND traversal MUST complete without unbounded recursion

#### Scenario: Version or depth is unsupported

- GIVEN a derivation uses an unknown version tag, exceeds the depth policy, requests no outputs, or uses unsupported execution semantics
- WHEN admission runs
- THEN Mantle MUST return a stable unsupported or limit blocker
- AND it MUST NOT calculate identity or mutate scheduler state

### Requirement: Registry and scheduler mutation require full admission

r[dynamic_derivation_admission.registry_boundary] The Worker MUST mutate the derivation registry, waiter graph, goal set, or scheduler only from a registry-ready dynamic derivation. Every earlier failure MUST leave those states unchanged.

#### Scenario: Registry-ready derivation is new

- GIVEN a registry-ready derivation has no existing path entry
- WHEN the Worker applies its registration plan
- THEN it MAY insert the derivation and create the required goal and waiter relationships
- AND the mutation result MUST reference the admitted full identity

#### Scenario: Path is already present with different identity

- GIVEN a registry-ready candidate maps to an existing path with different admitted identity
- WHEN the Worker evaluates duplicate admission
- THEN it MUST reject the collision before waiter or goal mutation
- AND insertion order or discovery timing MUST NOT select one identity

### Requirement: Native compatibility remains explicit

r[dynamic_derivation_admission.compatibility] Covered native dynamic derivations MUST preserve their existing Mantle BLAKE3 identity and configured logical store path when all parent facts are complete. Nix compatibility identity MUST remain outside this native admission core.

#### Scenario: Covered native fixture remains stable

- GIVEN a current accepted traditional fixture, the same configured prefix, and complete equivalent parent facts
- WHEN the staged core computes identity and path
- THEN the BLAKE3 identity and configured-prefix derivation path MUST match the accepted golden values
- AND registry planning MUST retain the covered behavior

#### Scenario: Nix digest enters native parent facts

- GIVEN a parent fact carries a Nix compatibility digest role where a native Mantle derivation digest is required
- WHEN native admission validates the fact
- THEN it MUST reject the wrong-domain digest before identity calculation
- AND no implicit byte-level conversion MAY make the roles interchangeable

### Requirement: Admission claims remain local

r[dynamic_derivation_admission.claim_boundary] Dynamic derivation evidence MUST limit its claim to the recorded parsing, semantic, identity-completeness, path, and registration-plan rules. It MUST NOT claim builder correctness, source trust, sandbox enforcement, output correctness, scheduler optimality, or release eligibility.

#### Scenario: Dynamic derivation is registered

- GIVEN a native dynamic derivation passes every admission stage and enters the registry
- WHEN Mantle reports the result
- THEN it MAY state that the recorded metadata and parent identity facts passed the declared admission policy
- AND it MUST retain all execution, output, trust, and release non-claims

#### Scenario: Admission evidence overclaims

- GIVEN a report promotes successful registration to builder safety, output correctness, hermeticity, reproducibility, or release readiness
- WHEN evidence validation runs
- THEN validation MUST fail with a deterministic non-claim violation
- AND the registration evidence MUST remain bounded to local admission facts
