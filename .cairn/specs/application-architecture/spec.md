# Application Architecture Specification

## Purpose

Defines the `application-architecture` capability.

## Requirements

### Requirement: The CLI composition root is mechanical

r[application_architecture.thin_composition_root] Mantle's root dispatcher MUST limit itself to CLI parsing, runtime-context construction, concrete adapter selection, application dispatch, final presentation, and exit-status selection. It MUST NOT own domain policy, trust decisions, state transitions, receipt construction, provider translation, or hidden effect execution.

#### Scenario: A command is dispatched

GIVEN valid CLI input and an admitted runtime context
WHEN the root dispatcher handles the command
THEN it MUST map input to one capability-scoped application operation and provide explicit adapters
AND application policy MUST remain in the owning core or application service.

#### Scenario: Domain policy enters the root dispatcher

GIVEN source adds route, retry, trust, package, store, remote, or lifecycle decision logic to the root dispatcher
WHEN the application architecture rail runs
THEN it MUST fail with the policy owner and source location
AND moving the logic into another root-only helper MUST NOT bypass the failure.

### Requirement: Application operations own their ports and contracts

r[application_architecture.application_owned_ports] Each Mantle application operation MUST own narrow commands, results, blockers, effect plans, and ports for genuine external capabilities. Port contracts MUST use Mantle-owned types and dependencies MUST point inward from adapters to application code and cores.

#### Scenario: An application operation needs external state

GIVEN an operation requires filesystem, process, network, store, clock, random, credential, or telemetry capability
WHEN the composition root constructs that operation
THEN it MUST provide an explicit narrow adapter through an application-owned port
AND the operation MUST NOT obtain the capability from ambient global state.

#### Scenario: A provider type enters an application contract

GIVEN a port or application command accepts a framework, vendor, transport, store-service, Clap, or presentation type
WHEN API-shape and dependency checks run
THEN they MUST reject the outward dependency
AND translation MUST remain in the inbound or outbound adapter.

### Requirement: Errors retain ownership until presentation

r[application_architecture.typed_error_ownership] Functional cores MUST return typed domain blockers, application ports MUST return capability-specific errors, and adapters MUST retain provider failure details. Mantle MUST map these values to `RunError`, human text, JSON, and exit status only at the CLI presentation boundary.

#### Scenario: A domain decision rejects input

GIVEN a core rejects an admitted command because a policy invariant fails
WHEN the application and CLI report the failure
THEN the typed domain blocker MUST remain available until presentation mapping
AND a generic string or provider error MUST NOT replace its semantic class.

#### Scenario: An adapter fails

GIVEN a filesystem, process, network, store, or telemetry adapter fails while executing an accepted effect
WHEN the application records the observation
THEN it MUST preserve the capability and effect identity with the bounded adapter failure
AND it MUST NOT reclassify the failure as domain rejection or successful execution.

### Requirement: Effect plans and observations remain distinct

r[application_architecture.effect_observation_boundary] Functional cores MUST return typed bounded effect plans without executing effects. Application shells MUST execute plans through explicit ports and MUST use typed observations before they report terminal effect outcomes.

#### Scenario: Core authorizes an effect

GIVEN admitted facts authorize a filesystem, process, network, store, or publication effect
WHEN the core returns its decision
THEN the result MUST identify the effect, authority class, limits, and expected observation
AND it MUST not claim that the effect started or succeeded.

#### Scenario: Effect observation contradicts the plan

GIVEN an adapter returns an observation with the wrong effect identity, authority, output, or limit usage
WHEN the application classifies it
THEN it MUST reject the observation or report an execution blocker
AND it MUST not construct successful evidence from the mismatch.

### Requirement: Application dependency direction is maintained

r[application_architecture.dependency_guard] Mantle MUST maintain deterministic positive and negative architecture checks for core purity, application-owned ports, adapter dependency direction, explicit composition roots, typed error ownership, and presentation separation.

#### Scenario: Accepted topology passes

GIVEN cores, application operations, ports, adapters, and presentation modules follow the declared dependency direction
WHEN the maintained architecture rail runs
THEN it MUST accept the topology and report the checked owners and boundaries
AND it MUST include the new core crates in host and `wasm32-unknown-unknown` checks where declared.

#### Scenario: Infrastructure leaks inward

GIVEN a core or application contract imports CLI, Snix, filesystem, process, async-runtime, environment, clock, random, network, provider, or rendering authority outside an accepted adapter
WHEN the architecture rail runs
THEN it MUST fail with the authority class, owner, and dependency path
AND source aliases or re-exports MUST NOT bypass the failure.
