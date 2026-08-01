# Mantle Bounded Tree adoption

## ADDED Requirements

### Requirement: Completed immutable prerequisite

r[mantle.bounded_tree_adoption.prerequisite] Mantle MUST adopt only a completed `bounded-tree` release pinned to one immutable Radicle revision.

#### Scenario: Reviewed dependency is available
- GIVEN passing `bounded-tree` completion evidence and one immutable Radicle revision
- WHEN Mantle dependency admission runs
- THEN the exact revision MUST be accepted without a sibling path or fallback

#### Scenario: Shared repository is incomplete
- GIVEN an unchecked implementation task, failed gate, mutable revision, or missing Radicle source
- WHEN Mantle dependency admission runs
- THEN adoption MUST remain blocked

### Requirement: Release copy parity

r[mantle.bounded_tree_adoption.release_copy] Mantle MUST preserve release tree limits, accepted plans, destination bytes, modes, links, and blocker classes through the shared adapter.

#### Scenario: Valid release tree is copied
- GIVEN a valid release tree fixture
- WHEN old and shared-backed paths run
- THEN their accepted plan and destination tree MUST match

#### Scenario: Invalid release tree is rejected
- GIVEN a malformed, exceeded, changed, linked, special, or colliding release tree fixture
- WHEN old and shared-backed paths run
- THEN their stable blocker classes MUST match

### Requirement: Frontend identity compatibility

r[mantle.bounded_tree_adoption.frontend_identity] Mantle MUST retain its versioned frontend root preimage and executable-bit semantics while consuming shared member facts.

#### Scenario: Frontend tree is stable
- GIVEN a valid frontend artifact tree
- WHEN both identity paths run
- THEN the ordered entries and final Mantle digest MUST match

#### Scenario: Shared crate assigns product identity
- GIVEN a proposed adapter that accepts a generic shared root digest as Mantle identity
- WHEN boundary review runs
- THEN the adapter MUST be rejected

### Requirement: Positive and negative parity gate

r[mantle.bounded_tree_adoption.parity] Mantle SHALL remove local mechanism code only after positive and negative dual-run fixtures pass.

#### Scenario: Complete parity passes
- GIVEN passing valid and invalid fixture comparisons
- WHEN cutover readiness is evaluated
- THEN duplicated mechanism code MAY be removed

#### Scenario: Any parity case differs
- GIVEN a plan, identity, destination, or failure-class mismatch
- WHEN cutover readiness is evaluated
- THEN local mechanism code MUST remain active

### Requirement: Mantle policy boundary

r[mantle.bounded_tree_adoption.boundary] Mantle MUST retain build, store, artifact, evidence, and release policy outside `bounded-tree`.

#### Scenario: Shared observations are accepted
- GIVEN valid shared member facts
- WHEN Mantle constructs product output
- THEN Mantle MUST apply its own product policy and evidence meaning

#### Scenario: Dependency evidence overclaims
- GIVEN evidence that promotes tree mechanics to build correctness or release eligibility
- WHEN evidence validation runs
- THEN validation MUST fail

### Requirement: Coupled rollback

r[mantle.bounded_tree_adoption.rollback] Mantle MUST record a rollback that restores the prior adapter and dependency state together.

#### Scenario: Adoption regression appears
- GIVEN a post-cutover compatibility regression
- WHEN rollback executes
- THEN Mantle MUST restore the recorded pre-adoption source and dependency state

#### Scenario: Only dependency state is restored
- GIVEN a rollback that leaves the new adapter with the old dependency state
- WHEN rollback validation runs
- THEN validation MUST fail
