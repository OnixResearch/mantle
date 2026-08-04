# Store Lifecycle Specification

## Purpose

Defines the `store-lifecycle` capability.

## Requirements

### Requirement: Store lifecycle decisions have compiler-enforced cores

r[mantle.store_lifecycle.core_boundary] Mantle MUST place selected garbage-collection and final-NAR repair decisions in separate host-capability-free core crates that consume normalized observations and return typed intents or classifications without store, filesystem, process, environment, clock, network, async-runtime, signing, or mutation authority.

#### Scenario: Core decisions use supplied observations

r[mantle.store_lifecycle.core_boundary.scenario.pure]
- GIVEN bounded normalized store lifecycle observations
- WHEN a GC or repair core decision runs
- THEN it MUST return deterministic data without querying a service, scanning a path, rendering a NAR, obtaining a key, writing state, or deleting an object

#### Scenario: Host or store authority enters a core

r[mantle.store_lifecycle.core_boundary.scenario.forbidden]
- GIVEN a core imports filesystem, async-runtime, store-handle, PathInfo-service, castore-service, signing, process, environment, clock, or network authority
- WHEN architecture checks run
- THEN they MUST fail with a deterministic diagnostic that names the authority class and dependency path

### Requirement: Garbage collection separates observation, decision, and mutation

r[mantle.store_lifecycle.gc_core] Mantle MUST compute retained closure, live and dead logical paths, retained sidecars and result references, orphan candidates, reclaim summaries, ordered mutation intents, and dry-run reports from explicit bounded observations before the shell performs destructive GC effects.

#### Scenario: Complete observations produce a GC plan

r[mantle.store_lifecycle.gc_core.scenario.accepted]
- GIVEN complete retained-root, reference-graph, PathInfo, object, sidecar, result-reference, path, and size observations within named bounds
- WHEN GC planning runs
- THEN it MUST return deterministic live, dead, retained, candidate, reclaim-summary, and ordered-intent data
- AND dry-run and execute shells MUST consume the same admitted plan

#### Scenario: Retention or reachability facts are incomplete

r[mantle.store_lifecycle.gc_core.scenario.rejected]
- GIVEN a retained root is missing, a live result reference lacks its target, graph facts conflict, an observation exceeds a bound, arithmetic overflows, or a concrete path cannot be mapped to the required logical role
- WHEN GC planning runs
- THEN it MUST reject before deletion, database replacement, or successful reclaim reporting

### Requirement: Final-NAR repair separates observation, decision, and mutation

r[mantle.store_lifecycle.repair_core] Mantle MUST decide current, repairable, and rejected final-NAR states, signature replacement disposition, sidecar disposition, and bounded report content from explicit recorded and observed facts before the shell signs or mutates store state.

#### Scenario: Stale final-NAR facts are repairable

r[mantle.store_lifecycle.repair_core.scenario.repairable]
- GIVEN one exact signed PathInfo has complete local content, valid store-path identity, fresh observed final-NAR facts, and compatible sidecar facts
- WHEN repair planning runs
- THEN it MUST return the accepted dry-run or execution intent while preserving store path, node, references, CA metadata, deriver, output bytes, and declared claim facts

#### Scenario: Repair facts do not establish authority

r[mantle.store_lifecycle.repair_core.scenario.rejected]
- GIVEN selection is ambiguous, local content is incomplete, identity conflicts, required signatures or sidecar facts are invalid, or execution observations contradict the plan
- WHEN repair admission or result classification runs
- THEN it MUST reject or report the exact failed phase
- AND it MUST NOT fabricate signing, persistence, rollback, or verification success

### Requirement: Store lifecycle extraction preserves compatibility

r[mantle.store_lifecycle.compatibility] Core extraction MUST preserve accepted GC candidates, retention behavior, dry-run and execute reports, mutation order, store paths, signatures, sidecars, final-NAR SHA-256 fields, Mantle-owned BLAKE3 identities, diagnostics, and CLI behavior unless a separate versioned change approves a difference.

#### Scenario: Accepted fixtures cross the new boundary

r[mantle.store_lifecycle.compatibility.scenario.accepted]
- GIVEN an accepted GC or repair fixture
- WHEN legacy and extracted decision paths process the same observations
- THEN candidate sets, classifications, intents, canonical reports, digest roles, and terminal outcomes MUST remain equal

#### Scenario: Faulted shell execution remains fail closed

r[mantle.store_lifecycle.compatibility.scenario.fault]
- GIVEN an admitted plan encounters a lock, scan, delete, database, stage, sign, persist, rollback, cleanup, or verification failure
- WHEN the shell reports the observation to the decision boundary
- THEN the final report MUST retain the accepted failure semantics and MUST NOT claim successful reclaim or repair

### Requirement: Store lifecycle core architecture is maintained

r[mantle.store_lifecycle.architecture_guard] Mantle MUST include the GC and repair cores in its maintained no-std inventory, ownership review, API-shape policy, Octet topology, and host plus `wasm32-unknown-unknown` checks.

#### Scenario: Core and shell topology is valid

r[mantle.store_lifecycle.architecture_guard.scenario.accepted]
- GIVEN both cores use owned alloc-compatible data and shell adapters depend inward
- WHEN canonical architecture checks run
- THEN they MUST evaluate both cores and the `crunch-store` adapters successfully

#### Scenario: Adapter convenience leaks inward

r[mantle.store_lifecycle.architecture_guard.scenario.adapter-leak]
- GIVEN borrowed std convenience, host paths, async traits, store types, or shell error types enter a public core API
- WHEN API-shape and dependency checks run
- THEN they MUST fail until the convenience remains in a std adapter

### Requirement: Store lifecycle evidence keeps local claims

r[mantle.store_lifecycle.claim_boundary] Mantle MUST limit store lifecycle core claims to deterministic decisions over the supplied observations and evaluated source topology.

#### Scenario: All core and shell checks pass

r[mantle.store_lifecycle.claim_boundary.scenario.non-claim]
- GIVEN GC, repair, compatibility, shell fault, and architecture checks pass
- WHEN evidence is summarized
- THEN it MUST NOT claim source trust, content correctness beyond measured facts, complete reachability beyond supplied observations, safe deletion, recovered historical signer authority, sandboxing, or release eligibility
