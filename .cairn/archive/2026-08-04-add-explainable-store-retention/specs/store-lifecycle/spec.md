# Store Lifecycle Delta

## ADDED Requirements

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
