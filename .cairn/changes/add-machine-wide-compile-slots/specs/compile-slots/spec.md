# Specification: Machine-wide compile slots

## ADDED Requirements

### Requirement: Declared slot authority

r[mantle.compile_slots.slot_authority] Mantle MUST expose a per-machine slot
authority that grants compile slots to requesting builds under a declared
policy naming total slots, per-build reservation bounds, and queue limits.

A compile routed through the driver seam MUST request a slot before starting
and MUST release it after the compiler exits. Grants and releases MUST be
recorded as scheduling receipts, distinct from build evidence.

#### Scenario: Concurrent builds respect the bound

- GIVEN two concurrent builds each requesting many compile slots under a
  total-slot policy
- WHEN compiles run
- THEN total concurrent compiles MUST NOT exceed the declared total and every
  grant MUST be recorded.

#### Scenario: Queue limit

- GIVEN a build whose pending slot requests exceed the declared queue limit
- WHEN further requests arrive
- THEN the authority MUST apply the declared policy, rejecting or
  de-prioritizing the request rather than growing without bound.

### Requirement: Degrade without the authority

r[mantle.compile_slots.degrade_without_authority] A build MUST NOT require
the slot authority. When the authority is absent or unreachable, compiles
MUST run immediately with the build's own scheduling, and the build MUST
succeed with a degraded-disposition receipt.

#### Scenario: Authority absent

- GIVEN a build with slot support enabled and no authority endpoint available
- WHEN the build runs
- THEN every compile MUST run without waiting and the build MUST succeed.

#### Scenario: Authority dies mid-build

- GIVEN a running build with granted slots and the authority terminating
- WHEN the build continues
- THEN held slots MUST be released or reclaimed safely and later compiles
  MUST proceed without the authority.

### Requirement: Bounded fairness

r[mantle.compile_slots.bounded_fairness] Slot grants MUST have bounded wait,
starvation controls, and per-build fairness so that no build monopolizes the
machine's compile capacity.

The fairness policy MUST be declared, versioned, and observable in
scheduling receipts. A waiting build MUST eventually receive slots under the
declared policy.

#### Scenario: Large build does not starve a small one

- GIVEN a build holding many slots and a second build waiting for its first
- WHEN the fairness policy applies
- THEN the second build MUST receive a slot within the declared bound.

#### Scenario: Grant wait is bounded

- GIVEN any request pattern within policy bounds
- WHEN a compile waits for a slot
- THEN the wait MUST remain within the declared bound or the request MUST be
  rejected with a typed reason.

### Requirement: Slots never influence outputs

r[mantle.compile_slots.no_output_influence] Slot enforcement MUST affect
scheduling only. Outputs, build receipts, and derivation identities MUST be
identical with and without the authority.

Scheduling receipts MUST be recorded outside the derivation graph and MUST
NOT be accepted as build evidence in strict lanes.

#### Scenario: Identity parity

- GIVEN the same derivation built once with the authority and once without
- WHEN outputs are compared
- THEN they MUST be byte-identical and derivation identities MUST match.

#### Scenario: Scheduling receipt offered as evidence

- GIVEN a review packet citing slot receipts as build evidence
- WHEN a strict lane evaluates it
- THEN the lane MUST reject the citation as scheduling state, not evidence.
