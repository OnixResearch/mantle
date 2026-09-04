## ADDED Requirements

### Requirement: Dynamic child authority is attenuated [r[dynamic_derivation_admission.authority_attenuation]]

Mantle MUST admit a dynamic child only when its effective effect authority is equal to or narrower than the producer's admitted authority ceiling. One pure deterministic decision MUST compare normalized Mantle-owned authority facts for native plans and traditional compatibility derivations. Child action shape MUST NOT grant authority by itself.

#### Scenario: An offline child inherits offline authority

- GIVEN an offline producer emits a child whose execution profile and effective effects require no additional authority
- WHEN dynamic child admission compares the parent ceiling and child request
- THEN admission MUST return an accepted child-authority decision
- AND the child MUST retain the producer's offline network boundary

#### Scenario: A fixed-output child attempts network escalation

- GIVEN an offline producer emits a traditional child that uses `builtin:fetchurl` with a fixed output
- WHEN dynamic child admission derives the child's effective network request
- THEN admission MUST fail with `dynamic-child-authority-widening`
- AND Mantle MUST NOT call a fetch service, insert the child, or create scheduler state

#### Scenario: Another authority field widens

- GIVEN a child requests broader environment, shell, setid, syscall, writable-prefix, host-path, substitution, or logical-store authority than its producer
- WHEN the pure attenuation decision runs
- THEN it MUST return ordered typed blockers for every widened class
- AND no child profile or registry-ready value MUST be produced

#### Scenario: Both dynamic lanes use one policy

- GIVEN equivalent child effects arrive through `mantle-plan-v1` and the traditional `.drv` compatibility lane
- WHEN both candidates reach authority admission
- THEN both MUST use the same Mantle-owned attenuation policy
- AND the compatibility lane MUST NOT receive a Nix-specific authority exception

### Requirement: Dynamic authority is bound to registration [r[dynamic_derivation_admission.authority_binding]]

Mantle MUST bind every accepted dynamic child to its admitted execution profile, parent ceiling identity, and domain-separated BLAKE3 attenuation-decision identity. Registry and scheduler mutation MUST require those values and MUST recheck the current parent identity before mutation.

#### Scenario: Accepted child reaches registry insertion

- GIVEN a child has an accepted attenuation decision and the current parent profile matches the decision
- WHEN the Worker applies the registry plan
- THEN the registry entry MUST contain the admitted child profile and decision identity
- AND later build-request creation MUST use that exact profile

#### Scenario: A dynamic path selects a default profile

- GIVEN a native or compatibility dynamic path attempts to register a child with an implicit `ExecutionProfile::native_compatibility()` default
- WHEN the registration boundary validates the request
- THEN registration MUST fail with a deterministic missing-authority-binding blocker
- AND the default profile MUST NOT replace the admitted child profile

#### Scenario: Parent authority changes before mutation

- GIVEN attenuation accepted a child under one parent profile identity
- AND the current parent profile or effective policy no longer matches that identity
- WHEN the Worker prepares registry or scheduler mutation
- THEN it MUST reject the stale decision
- AND registry, goals, waiters, fetch services, and scheduler state MUST remain unchanged

#### Scenario: Native version-one inheritance remains compatible

- GIVEN a valid `mantle-plan-v1` unit declares the existing `inherit` policy values
- WHEN the unit is admitted under an unchanged parent profile
- THEN Mantle MUST preserve the version-one wire bytes and canonical plan identity
- AND runtime registration MUST use the actual admitted parent policy instead of a new default profile
