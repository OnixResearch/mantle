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

### Requirement: Typed store retention policy

r[store_lifecycle.retention_policy] Mantle MUST decide root retention through a typed, versioned policy over explicit root, owner, generation, lease, closure, and supplied clock facts. Retention policy MUST use named bounds and MUST NOT use filesystem access time or hidden ambient state as retention authority.

#### Scenario: Project generation remains within policy

- **GIVEN** a project output root has valid owner, selector, generation, policy, and closure facts within the retained generation window
- **WHEN** the pure retention planner evaluates the root
- **THEN** it MUST return a keep decision with stable project-generation reason code
- **AND** equivalent facts MUST produce the same decision and ordering

#### Scenario: Lease facts are unsafe

- **GIVEN** a shell lease has malformed expiry, clock rollback, unknown owner, stale policy, or an exceeded renewal bound
- **WHEN** retention planning evaluates the lease
- **THEN** it MUST fail closed or quarantine the root as policy specifies
- **AND** it MUST NOT infer expiry from filesystem timestamps

### Requirement: Versioned root provenance

r[store_lifecycle.root_provenance] Every managed GC root MUST bind a logical path, root class, owner scope, policy identity, and applicable project, selector, generation, or lease facts. Legacy roots without these facts MUST remain protected and explicitly unclassified until migration or removal.

#### Scenario: Successful project build registers an output generation

- **GIVEN** a selected project root completes and its PathInfo and output admission succeed
- **WHEN** Mantle commits retention state
- **THEN** it MUST register the output under the canonical project, selector, generation, and policy identities
- **AND** it MUST NOT remove the previously retained generation until the same transition commits safely

#### Scenario: Legacy root is loaded

- **GIVEN** a path-only root record from the prior schema is valid but has no project or lease facts
- **WHEN** the root registry migrates it
- **THEN** Mantle MUST classify it as protected legacy unmanaged state
- **AND** it MUST NOT invent project ownership, expiry, or generation facts

### Requirement: Store usage report

r[store_lifecycle.usage_report] Mantle MUST report bounded observed store usage with retained, reclaimable, quarantined, unclassified, shared, and unknown classes. Totals MUST count shared content once, use checked arithmetic, and distinguish per-root inclusive bytes from unique bytes.

#### Scenario: Two roots share one closure member

- **GIVEN** two retained roots reach one shared PathInfo or castore object
- **WHEN** Mantle computes store usage
- **THEN** total retained bytes MUST count that content once
- **AND** per-root output MUST identify inclusive and unique values without claiming both roots uniquely own the bytes

#### Scenario: Byte observation is incomplete

- **GIVEN** a retained or candidate object cannot be read or measured within policy bounds
- **WHEN** usage reporting completes
- **THEN** Mantle MUST report unknown bytes and the stable observation blocker
- **AND** it MUST NOT treat unknown bytes as zero

### Requirement: Explained retention and collection

r[store_lifecycle.gc_explanation] Mantle MUST expose deterministic bounded explanations for direct roots, retained closure members, expired roots, quarantined state, and GC candidates. Explanations MUST identify stable reason codes, owner and policy facts when present, and bounded retaining-root links.

#### Scenario: Path remains live

- **GIVEN** a store path is reachable from one or more retained roots
- **WHEN** an operator asks for roots or a GC plan with explanations
- **THEN** Mantle MUST identify the direct or transitive retention reason and bounded retaining roots
- **AND** it MUST distinguish explicit pin, project generation, lease, bootstrap, self-build, and legacy classes

#### Scenario: Path becomes a candidate

- **GIVEN** no accepted root reaches a valid PathInfo and policy authorizes removal
- **WHEN** Mantle creates an explained GC plan
- **THEN** the plan MUST identify the removal reason and observed reclaimable bytes or unknown-byte blocker
- **AND** plan creation MUST NOT mutate store state

### Requirement: Plan-bound GC execution

r[store_lifecycle.safe_gc_execution] Mantle MUST make ordinary GC planning non-mutating and MUST require explicit execution against an accepted plan identity. Execution MUST re-observe root, policy, PathInfo, closure, and relevant storage facts and MUST reject drift before deletion.

#### Scenario: Reviewed plan remains current

- **GIVEN** an accepted GC plan and unchanged bound state
- **WHEN** an operator explicitly executes the plan
- **THEN** Mantle MAY remove only the plan-authorized unreferenced state under the store mutation lock
- **AND** it MUST record completed and failed operations without claiming full success after partial failure

#### Scenario: Root changes after planning

- **GIVEN** a root, lease, policy, PathInfo, closure, or relevant storage fact changes after plan creation
- **WHEN** plan execution revalidates the preimage
- **THEN** Mantle MUST reject the stale plan before deletion
- **AND** it MUST require a new plan

### Requirement: Store lifecycle validation

r[store_lifecycle.retention_validation] Store lifecycle changes MUST include positive, negative, interruption, corruption, overflow, stale-plan, clock, shared-closure, and deletion-failure fixtures plus focused core, shell, migration, and CLI validation.

#### Scenario: Positive lifecycle rail passes

- **GIVEN** valid pins, project generations, leases, roots, closures, usage observations, and an unchanged execution plan
- **WHEN** the focused validation rail runs
- **THEN** planning and execution fixtures MUST produce their expected deterministic decisions
- **AND** the final state MUST retain every policy-live path

#### Scenario: Negative fixture rejects for the intended reason

- **GIVEN** a fixture contains corrupt state, unsafe clock facts, overflow, stale plan, changed root, missing closure data, symlink substitution, or deletion failure
- **WHEN** the focused validation rail runs
- **THEN** it MUST produce the expected stable failure class
- **AND** an unrelated failure MUST NOT count as valid rejection evidence
