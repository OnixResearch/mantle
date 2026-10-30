# Specification: Declared remote worker demand

## ADDED Requirements

### Requirement: Unmet requirements publish a bounded demand fact

r[build_scheduling.declared_worker_demand] When a quantified job cannot be
placed under current admitted worker facts, Mantle MUST publish one bounded
demand fact naming the required resource vector, the job identity, the policy
identity, the requirement identity, and the blocker reason.

Demand MUST deduplicate by requirement identity, MUST be bounded in count, and
MUST retract when the job is placed, cancelled, times out, or changes its
requirement.

The building plane derives the fact. The coordination daemon from ADR 0080
MUST serve demand facts to subscribers and MUST NOT derive them. A build MUST
NOT require the daemon to report its blocker.

#### Scenario: Unmet demand is published once

- GIVEN a quantified job with no admitted worker that fits
- WHEN the placement planner evaluates it across several rounds
- THEN exactly one demand fact MUST exist for that requirement identity
- AND it MUST name the blocker reason from the plan result

#### Scenario: Placement retracts demand

- GIVEN a published demand fact for an unplaced job
- WHEN a supervisor-registered worker satisfies the requirement and the job is
  placed
- THEN the demand fact MUST retract

### Requirement: Demand never fabricates capacity

r[build_scheduling.demand_without_capacity_inference] A demand fact MUST NOT
create a worker, lease, reservation, or placement, MUST NOT change admission
or scheduling preference, and MUST NOT derive capacity from concurrency,
worker presence, or history.

A demand fact MAY be read by an operator-side supervisor. Any worker it starts
MUST enter Mantle through ordinary registration and admission.

#### Scenario: Demand cannot grant a lease

- GIVEN a published demand fact and no admitted worker
- WHEN a placement round runs
- THEN no lease or reservation MUST be created from the fact
- AND the job MUST remain blocked

#### Scenario: Worker registration resolves demand normally

- GIVEN a published demand fact
- WHEN a new worker registers with a satisfying verified inventory
- THEN the job MUST be placed under the existing admission rules
- AND the demand fact MUST retract
