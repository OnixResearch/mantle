# Remote Builds Hexagonal Architecture Delta

## ADDED Requirements

### Requirement: Remote execution has a strict functional core

r[remote_builds.hexagonal_core] Mantle MUST keep remote protocol admission, normalized identities, state transitions, fencing, retry classification, resource and transfer decisions, output-admission inputs, deterministic events, effect planning, and receipt preimages in a `no_std + alloc` functional core over explicit bounded values.

#### Scenario: Equivalent remote observations replay

GIVEN two runs supply equivalent admitted request, state, policy, resource, transfer, trust, and execution observations in different collection orders
WHEN the remote core applies the transition sequence
THEN both runs MUST return the same states, events, blockers, effects, and receipt preimages
AND no filesystem, process, network, environment, clock, random, async-runtime, store, credential, or presentation authority may enter the core.

#### Scenario: Host authority enters the remote core

GIVEN the remote core imports or exposes Snix, `crunch-store`, Tokio, filesystem paths, processes, network clients, environment state, clocks, random sources, credentials, or renderers
WHEN the architecture rail runs
THEN it MUST fail with the authority class and dependency path
AND an adapter wrapper MUST NOT make the inward dependency acceptable.

### Requirement: Remote application ports use Mantle-owned contracts

r[remote_builds.application_owned_ports] Remote application ports MUST use Mantle-owned commands, observations, effects, outcomes, blockers, and capability errors. Snix PathInfo, build request, build result, substitution report, provider SDK, transport, and CLI error types MUST remain in adapters.

#### Scenario: A remote output reaches admission

GIVEN a remote adapter receives protocol and Snix output records
WHEN it supplies output facts to the remote application
THEN it MUST translate them into admitted Mantle-owned output facts
AND the application port MUST NOT expose the originating vendor or transport type.

#### Scenario: A vendor type enters a port signature

GIVEN a new or changed remote application port accepts or returns a Snix, store, transport, provider, or CLI-owned type
WHEN dependency and API-shape checks run
THEN they MUST reject the port signature
AND compatibility translation MUST remain in the owning adapter.

### Requirement: Remote effects are planned before execution

r[remote_builds.remote_effect_plans] The remote core MUST return bounded typed effects for transport, attempt persistence, lease changes, input transfer, executor launch, output admission, and telemetry publication. The imperative shell MUST execute each effect through its capability port and return a typed observation before the next dependent transition.

#### Scenario: Accepted request needs executor work

GIVEN an admitted remote request, current state, and policy authorize one executor attempt
WHEN the core plans the next transition
THEN it MUST return the next state, deterministic events, and one bounded executor effect
AND it MUST NOT claim that the executor launched or completed.

#### Scenario: Effect execution fails

GIVEN the core returned an accepted effect and the selected adapter fails
WHEN the shell submits the failed observation to the core
THEN the core MUST classify the observation through the declared transition and retry policy
AND the shell MUST NOT fabricate a success event or skip the failed observation.

### Requirement: Remote extraction preserves accepted compatibility

r[remote_builds.hexagonal_compatibility] Hexagonal extraction MUST preserve accepted remote wire bytes, canonical identities, state transitions, fencing, retry, transfer, trust, resource, output-admission, receipt, diagnostic, and CLI behavior unless a separate versioned change authorizes a difference.

#### Scenario: Accepted fixture crosses the new boundary

GIVEN an accepted remote request, observation sequence, and adapter fixture
WHEN legacy and extracted paths process equivalent inputs
THEN their wire output, decisions, events, effects, terminal outcome, and receipt material MUST remain equal
AND any intentional difference MUST require a separately reviewed compatibility change.

#### Scenario: Malformed or stale input crosses the boundary

GIVEN a malformed request, stale fence, untrusted output, exceeded bound, or failed adapter observation
WHEN the extracted path processes it
THEN it MUST retain the accepted fail-closed reason and terminal behavior
AND an unrelated failure MUST NOT count as compatibility evidence.
