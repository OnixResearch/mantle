## ADDED Requirements

### Requirement: INV-1 Machine records

The inventory MUST contain a `machines` field whose keys are machine names and
whose values are machine records.
ID: systemconfig.inventory.inv1

Each machine record MUST contain a `system` string and MAY contain a `class`
field selecting the default assembler backend. When `class` is absent, it
defaults to `"nixos"`. Machine records MAY contain additional metadata fields,
and the pipeline MUST pass the full record through to the assembler.

#### Scenario: Inventory passes through extra machine metadata
ID: systemconfig.inventory.inv1.scenario

- GIVEN a machine record containing `system`, `class`, and extra metadata
- WHEN the inventory is deserialized
- THEN the record remains available to the assembler with the extra metadata
  intact

### Requirement: INV-2 Service instances

The inventory MUST contain a `services` field whose keys are module identities
(the same stable names derived from module filename stems) and whose values are
service records.
ID: systemconfig.inventory.inv2

Each service record MUST contain an `instances` array. Each instance MUST name
`machine` and `role`, and MAY include `settings` and `tags`.

#### Scenario: Inventory records one service with multiple instances
ID: systemconfig.inventory.inv2.scenario

- GIVEN a `services.nginx` record with two instances
- WHEN the inventory is deserialized
- THEN the `nginx` service key identifies the target module by its module
  filename stem
- AND both instances are preserved with their machine, role, and optional
  settings fields

### Requirement: INV-3 Cross-reference validation boundary

The inventory specification MUST distinguish structural validation from
cross-reference validation.
ID: systemconfig.inventory.inv3

Structural validation belongs to the inventory contract. Cross-reference checks
that require loaded module metadata, such as matching service names, role names,
and machine names, MUST be deferred to the evaluator stage.

#### Scenario: Structural inventory validation does not require loaded modules
ID: systemconfig.inventory.inv3.scenario

- GIVEN a structurally valid inventory file
- WHEN the inventory contract is applied before module loading
- THEN the structural validation succeeds without consulting module metadata
- AND later cross-reference checks remain the evaluator's responsibility

### Requirement: INV-4 Embedded Nickel contract

The inventory structural contract MUST ship with crunch's embedded Nickel
stdlib.
ID: systemconfig.inventory.inv4

`crunch system eval` and `crunch system build` MUST apply `crunch.Inventory`
at the Nickel boundary themselves before Rust-side deserialization and limit
validation run. User inventories therefore MAY be plain pure-data Nickel
records and are NOT required to spell the merge explicitly in the file.
After that pipeline-applied contract merge succeeds, later pipeline stages
MUST consume a validated pure-data inventory value containing only the
`machines` and `services` records defined by INV-1 and INV-2, rather than an
unevaluated Nickel expression. If the pipeline-applied contract merge fails,
the command MUST treat the result as a fatal inventory validation error before
any module loading or machine evaluation begins.

#### Scenario: Inventory contract is available from the embedded stdlib
ID: systemconfig.inventory.inv4.scenario

- GIVEN a plain pure-data user inventory evaluated through crunch's embedded
  stdlib
- WHEN `crunch system eval` or `crunch system build` applies
  `crunch.Inventory` at the pipeline boundary
- THEN the structural contract is resolved without needing external files
- AND later pipeline stages receive a validated pure-data inventory value
- AND a contract failure would stop the command before any module loading or
  machine evaluation begins

### Requirement: INV-5 Fixed limits

The inventory MUST enforce configurable maximums of 4096 machines, 4096
instances per service, and 65536 total instances.
ID: systemconfig.inventory.inv5

Exceeding any limit MUST fail fast with a diagnostic naming the limit.

#### Scenario: Inventory rejects too many machines
ID: systemconfig.inventory.inv5.scenario

- GIVEN an inventory declaring more than 4096 machines
- WHEN inventory validation runs
- THEN validation fails before module evaluation begins
- AND the diagnostic names the machine-count limit

### Requirement: INV-6 Inventory is pure data

The inventory MUST remain a pure Nickel data document.
ID: systemconfig.inventory.inv6

The inventory MUST NOT import module files, call module `impl` functions, or
otherwise depend on system-module execution semantics.

#### Scenario: Inventory does not execute module code
ID: systemconfig.inventory.inv6.scenario

- GIVEN an inventory file submitted to `crunch system eval`
- WHEN the inventory is loaded
- THEN only inventory data is evaluated at that step
- AND no module `impl` function is called during inventory loading
