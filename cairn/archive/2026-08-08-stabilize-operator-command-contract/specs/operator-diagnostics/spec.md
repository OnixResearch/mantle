# Operator Diagnostics Delta

## ADDED Requirements

### Requirement: Canonical command catalog contract

r[operator_diagnostics.command_catalog_contract] Mantle MUST maintain one deterministic command catalog that links the executable parser graph to reviewed operator policy for public command paths, aliases, flags, support tiers, mutation classes, network classes, exit classes, and machine-output contracts.

#### Scenario: Parser and policy agree

- **GIVEN** the Clap graph and reviewed operator inventory describe the same supported command surface
- **WHEN** Mantle builds the command catalog
- **THEN** it MUST emit one deterministic ordered catalog
- **AND** equivalent parser and policy facts MUST produce the same catalog identity

#### Scenario: Command behavior drifts

- **GIVEN** a command, alias, flag, exit class, or machine-output promise exists on only one side of the parser and policy boundary
- **WHEN** command-contract validation runs
- **THEN** it MUST fail with a stable drift class
- **AND** it MUST NOT publish updated command documentation

### Requirement: Compatibility surface inventory

r[operator_diagnostics.compatibility_surface_inventory] Mantle MUST classify every retained operator-facing compatibility identifier with an owner, exact role, compatibility state, supported operations, migration reference, and removal gate. New operator prose and remediation MUST use canonical Mantle commands unless an exact compatibility role requires another spelling.

#### Scenario: Existing compatibility identifier remains readable

- **GIVEN** an existing serialized schema, project file, environment input, command alias, or sidecar is classified as readable compatibility
- **WHEN** a supported consumer supplies that identifier
- **THEN** Mantle MUST preserve the recorded read behavior
- **AND** diagnostics MUST identify the compatibility state without presenting the identifier as the preferred new surface

#### Scenario: Unclassified legacy identifier appears

- **GIVEN** new code, help, docs, examples, diagnostics, or generated data introduce an unclassified legacy product identifier
- **WHEN** compatibility validation runs
- **THEN** validation MUST fail with the file or command context and required owner action
- **AND** it MUST NOT silently add the identifier to the compatibility inventory

### Requirement: Structured remediation actions

r[operator_diagnostics.structured_remediation] Stable Mantle diagnostics MUST provide bounded structured remediation actions with stable diagnostic code, phase, safe subject, ordered next actions, mutation class, network class, and required preconditions. Human and JSON renderers MUST consume the completed remediation decision and MUST NOT invent commands during rendering.

#### Scenario: Failure has one safe next action

- **GIVEN** normalized failure facts match a supported remediation rule
- **WHEN** Mantle renders human or JSON diagnostics
- **THEN** both forms MUST identify the same diagnostic code and ordered next action
- **AND** the action MUST state whether it can mutate files, store state, or contact the network

#### Scenario: Failure contains secret-bearing context

- **GIVEN** raw failure context includes bearer material, private key paths, secret environment values, or unbounded arguments
- **WHEN** remediation is classified and rendered
- **THEN** the structured record MUST omit or redact the protected values
- **AND** no action command MUST embed the protected content

### Requirement: Canonical operator workflow

r[operator_diagnostics.canonical_operator_workflow] Mantle MUST publish one short canonical workflow derived from the accepted command catalog. The workflow MUST cover diagnosis, project inspection or refresh, build planning, local or remote realization, and evidence inspection while naming side effects and platform limits.

#### Scenario: New operator follows the canonical path

- **GIVEN** a supported project and host or an eligible remote route
- **WHEN** an operator follows the published canonical workflow
- **THEN** each documented command MUST exist with the shown options and ordering
- **AND** each step MUST identify mutation, network, and platform behavior before execution

#### Scenario: Host cannot execute local builds

- **GIVEN** the client host lacks a supported local executor
- **WHEN** the canonical workflow reaches realization planning
- **THEN** documentation and diagnostics MUST identify an eligible remote route or an explicit unsupported blocker
- **AND** they MUST NOT direct the operator to install Linux-only execution tools as the only remedy when remote realization is eligible

### Requirement: Command contract validation

r[operator_diagnostics.command_contract_validation] Mantle MUST validate command catalogs, generated docs, remediation records, stdout contracts, exit classes, compatibility states, and redaction with positive and negative fixtures before accepting operator-surface changes.

#### Scenario: Complete operator contract passes

- **GIVEN** parser, policy, docs, diagnostics, machine output, and compatibility fixtures agree
- **WHEN** the focused operator validation rail runs
- **THEN** all positive fixtures MUST pass
- **AND** generated catalog and documentation identities MUST match checked-in expectations

#### Scenario: One boundary regresses

- **GIVEN** a fixture contains stale help, wrong exit behavior, stdout pollution, an undocumented alias, a secret leak, or an unsafe remediation command
- **WHEN** the focused operator validation rail runs
- **THEN** the negative fixture MUST fail with its expected stable class
- **AND** no unrelated failure MAY count as evidence that the boundary rejected correctly
