# Specification: GC planning parity

## ADDED Requirements

### Requirement: Exact kernel import

r[mantle.gc_plan.kernel_import] Mantle MUST import the Terrane reclaim-plan
kernel and its tests unchanged from a pinned revision, MUST check their exact
BLAKE3 identities, and MUST compile the kernel only for tests.

#### Scenario: Vendored kernel drifts

- GIVEN a vendored kernel file whose bytes differ from the recorded identity
- WHEN the importer check runs
- THEN it MUST fail and name the drifted member.

### Requirement: Planning parity

r[mantle.gc_plan.planning_parity] For every enumerated store, `plan_gc` MUST
plan exactly when the kernel plans, MUST produce the kernel's retained paths,
candidate paths, deletion intents, protected base paths, and retaining roots,
and each rejection MUST name a violation the kernel also finds.

#### Scenario: Reachability walk drops a reference

- GIVEN a change that skips a reference while computing retained paths
- WHEN the parity rail runs
- THEN it MUST fail and report the store.

### Requirement: Checked output

r[mantle.gc_plan.checked_output] The kernel's verified `check_plan` MUST
accept every plan `plan_gc` produces in the enumerated space.

#### Scenario: Mantle plans a live path for deletion

- GIVEN a `plan_gc` result whose candidates include a path reachable from a
  root
- WHEN the rail applies `check_plan`
- THEN the check MUST fail.
