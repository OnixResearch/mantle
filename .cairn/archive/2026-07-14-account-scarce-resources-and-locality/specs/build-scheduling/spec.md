# Build Scheduling Specification

## Purpose

Derive deterministic resource eligibility and content-locality preference from bounded concrete facts.

## ADDED Requirements

### Requirement: Quantified resources are admitted before scheduling preference
r[build_scheduling.quantified_resource_admission]

Mantle MUST validate provider-neutral quantified action requirements and worker capacities for CPU, memory bytes, scratch bytes, accelerator classes/counts, and bounded named token pools before route preference. It MUST use checked arithmetic and current fenced reservations to determine hard resource eligibility. Dynamic capacity and scheduling-only quantities MUST NOT change action identity unless an explicit semantic action field declares that they affect execution meaning.

#### Scenario: Compatible capacity is reservable

- GIVEN an eligible action declares bounded resource quantities and a worker has matching unreserved capacity and semantic capabilities
- WHEN Mantle plans resource admission
- THEN it MUST return a deterministic reservation plan and normalized resource-fit class
- AND applying that plan MUST leave every remaining capacity nonnegative and within declared totals.

#### Scenario: Aggregate overcommit is rejected

- GIVEN individually valid jobs collectively request more CPU, memory, scratch, accelerator, or named-token capacity than remains
- WHEN Mantle plans another assignment
- THEN it MUST reject that worker before dispatch with a stable resource reason
- AND scheduler priority, locality, age, or provider response order MUST NOT authorize overcommit.

#### Scenario: Scheduling metadata does not silently alter action identity

- GIVEN two executions differ only in scheduling-only reservation quantities or dynamic worker availability
- WHEN Mantle computes their action refs
- THEN those facts MUST NOT change action identity
- AND any platform, toolchain, accelerator, sandbox, or other semantic difference MUST remain an explicit action-identity input.

### Requirement: Locality placement uses receiver-verified content facts
r[build_scheduling.verified_locality_placement]

Mantle MUST derive content-locality and transfer-cost preference from bounded receiver-verified object-presence and missing-set facts bound to the requested input manifest and policy. Unverified worker advertisements, stale probes, expected CA paths, or mutable cache claims MUST NOT establish full locality or zero transfer. Locality MAY rank eligible workers but MUST NOT bypass capability, trust, upload privacy, network, store-prefix, or hard resource policy.

#### Scenario: Verified local content improves placement

- GIVEN two workers satisfy every hard eligibility check
- AND one receiver verifies a more complete demanded input set with a lower bounded transfer class for the same manifest and policy
- WHEN the existing scheduler ranks the eligible candidates
- THEN it SHOULD prefer that worker according to configured field precedence
- AND evidence MUST identify normalized verified counts/bytes and freshness basis without unbounded object lists.

#### Scenario: Stale locality is rejected

- GIVEN a locality observation names the wrong manifest, policy, worker generation, or content that receiver probes no longer verify
- WHEN Mantle derives scheduling facts
- THEN it MUST reject or downgrade that observation with a stable reason
- AND it MUST NOT report `FullyPresent` or zero transfer from the stale claim.

#### Scenario: Locality cannot create eligibility

- GIVEN a worker has all input content but lacks output trust, upload permission, required semantic capability, store-prefix match, or reservable resources
- WHEN placement runs
- THEN the worker MUST remain ineligible
- AND no locality or transfer preference may cause dispatch.
