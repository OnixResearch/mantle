## ADDED Requirements

### Requirement: Distributed realization has an infrastructure-free decision core

r[realization_routing.distributed_core_boundary] Distributed realization keys, route eligibility, ranking, reuse, fallback, observation admission, and outcome decisions MUST use Mantle-owned bounded values without async calls, provider ports, or concrete infrastructure types.

#### Scenario: Vendor type enters the declared core boundary

r[realization_routing.distributed_core_boundary.scenario.vendor_type]
- GIVEN a distributed decision API exposes a Snix, store implementation, runtime, process, or provider type
- WHEN the distributed boundary gate runs
- THEN the gate MUST fail
- AND an adapter MUST project that value into a Mantle-owned observation before core evaluation.

#### Scenario: Core decision calls an external capability

r[realization_routing.distributed_core_boundary.scenario.effect_call]
- GIVEN route or outcome evaluation needs current external facts
- WHEN the pure decision runs
- THEN those facts MUST arrive as explicit bounded inputs
- AND the decision MUST return a blocker or effect plan instead of calling the external capability.

### Requirement: The distributed application shell owns orchestration

r[realization_routing.distributed_shell_ownership] Application-owned ports and a visible composition root MUST own resolver order, service calls, time, cancellation, retries, observation recording, and concrete adapter selection.

#### Scenario: Selected provider is unavailable

r[realization_routing.distributed_shell_ownership.scenario.unavailable]
- GIVEN the selected resolver or realizer is unavailable
- WHEN the shell records that observation
- THEN the core MUST decide the typed blocked, retry, fallback, or unknown disposition from explicit policy
- AND no adapter or core function MUST select another provider implicitly.

### Requirement: Distributed boundary validation covers success and failure

r[realization_routing.distributed_boundary_validation] Maintained tests MUST cover positive and negative core, shell, and adapter behavior for ordering, denial, malformed observations, stale identity, unavailable services, timeout, cancellation, ambiguity, and fallback policy.

#### Scenario: Only successful adapter behavior is tested

r[realization_routing.distributed_boundary_validation.scenario.missing_negative]
- GIVEN a distributed adapter has a successful fixture but lacks a required failure class
- WHEN distributed readiness runs
- THEN readiness MUST fail with the missing class
- AND successful execution MUST NOT substitute for negative boundary evidence.
