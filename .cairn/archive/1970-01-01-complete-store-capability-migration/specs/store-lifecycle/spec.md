# Store Lifecycle Capability Migration Delta

## ADDED Requirements

### Requirement: Store authority is exposed through narrow capabilities

r[store_lifecycle.capability_only_access] Mantle MUST keep raw PathInfo, directory, blob, signing, overlay, HTTP, database, and mutable session services inside the `crunch-store` shell. Callers outside that shell MUST use operation-specific capability values with Mantle-owned requests, results, and errors.

#### Scenario: A transfer shell reads admitted objects

GIVEN the remote-transfer shell needs bounded object facts for one admitted transfer
WHEN it requests those facts from the store
THEN it MUST use a transfer-specific capability with named high-level operations
AND it MUST NOT obtain a broad `StoreHandle` or raw Snix service.

#### Scenario: A caller requests unrelated store authority

GIVEN a caller holds an output-lookup capability
WHEN source code attempts to mutate roots, publish action results, sign PathInfo, or access raw object services through that value
THEN compilation or the deterministic architecture rail MUST fail
AND the caller MUST acquire a separately owned capability at the composition boundary.

### Requirement: Output publication uses an explicit effect plan

r[store_lifecycle.publication_effect_plan] After local output persistence and admission succeed, Mantle MUST return a bounded publication effect plan before any configured external publisher executes. The application shell MUST record each publisher result as a typed observation and MUST keep local admission truth separate from publication success.

#### Scenario: Admitted output has configured publishers

GIVEN an output has passed local persistence, signature, identity, and admission checks
AND one or more configured publishers apply
WHEN the store application completes local admission
THEN it MUST return the admitted output and an ordered bounded publication effect plan
AND the plan MUST NOT claim that any publisher executed.

#### Scenario: A publisher fails after local admission

GIVEN local output admission succeeded and one publication effect was attempted
WHEN the publisher returns a transport, authorization, or provider failure
THEN Mantle MUST retain the truthful local admission result and record a failed publication observation
AND it MUST NOT report that publisher as successful or erase the local output.

### Requirement: Store capability topology is compiler-enforced

r[store_lifecycle.capability_architecture_guard] Mantle MUST maintain positive and negative compiler, dependency, and source checks for store capability reachability. The rail MUST reject new broad-handle consumers, raw-service escape, unrelated methods on one capability, vendor types in application ports, or writable base-store authority.

#### Scenario: Accepted capability topology passes

GIVEN store callers depend only on their declared capability values and adapter projections remain inside the store shell
WHEN the architecture rail runs
THEN it MUST accept the topology and identify every allowed broad-handle compatibility owner
AND the allowlist MUST shrink to zero external consumers before this change completes.

#### Scenario: Raw service escapes through a new helper

GIVEN a helper outside the store shell returns or stores a raw PathInfo, directory, blob, signing, overlay, or database service
WHEN the architecture rail evaluates the dependency and source topology
THEN it MUST fail with the caller, service class, and dependency path
AND a source-string alias or wrapper MUST NOT bypass the failure.
