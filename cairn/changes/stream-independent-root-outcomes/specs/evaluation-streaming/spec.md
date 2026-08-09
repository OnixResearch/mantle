# Evaluation Streaming Specification

## ADDED Requirements

### Requirement: Complete selected-root outcomes

r[evaluation_streaming.complete_root_outcomes] Mantle MUST produce exactly one terminal outcome for every admitted selected root. A root-scoped error MUST NOT suppress evaluation of an independent selected root.

#### Scenario: One root fails and one root succeeds

- **GIVEN** two admitted selected roots and a root-scoped evaluation error for the first root
- **WHEN** Mantle evaluates the selected root set
- **THEN** Mantle MUST continue evaluation of the independent second root
- **AND** the terminal summary MUST contain exactly one outcome for each root

#### Scenario: Terminal observation repeats

- **GIVEN** one selected root already has a terminal outcome
- **WHEN** another terminal observation arrives for that root
- **THEN** the pure outcome transition MUST reject the duplicate
- **AND** the summary MUST NOT contain two outcomes for one root

### Requirement: Failure scope controls dispatch

r[evaluation_streaming.failure_scope] Mantle MUST classify an evaluation failure as root-scoped, shared fatal, cancellation, or coordinator failure before that failure changes dispatch. Only a root-scoped failure MAY permit independent root dispatch to continue.

#### Scenario: Root expression is invalid

- **GIVEN** one root fails during forcing without invalidating shared source, imports, evaluator cohort, protocol, or coordinator state
- **WHEN** failure-scope classification runs
- **THEN** the failure MUST be classified as root-scoped
- **AND** independent admitted roots MAY continue

#### Scenario: Shared evaluator initialization fails

- **GIVEN** evaluator initialization fails before any root can be forced safely
- **WHEN** failure-scope classification runs
- **THEN** the failure MUST be classified as shared fatal
- **AND** every selected root without an outcome MUST receive a stable not-started terminal class

### Requirement: Versioned bounded event stream

r[evaluation_streaming.versioned_event_stream] Mantle MUST provide a versioned NDJSON stream with bounded `run-start`, `root-discovered`, `root-terminal`, and `run-summary` records. Stream-mode stdout MUST contain only complete machine records, while human diagnostics MUST use stderr.

#### Scenario: Stream completes successfully

- **GIVEN** a supported stream schema and a bounded selected-root set
- **WHEN** evaluation completes
- **THEN** Mantle MUST emit one complete JSON object per line
- **AND** the final machine record MUST be one `run-summary`

#### Scenario: Record exceeds a named bound

- **GIVEN** a root diagnostic or event field exceeds its configured byte or collection bound
- **WHEN** Mantle constructs the machine record
- **THEN** it MUST apply the declared truncation or rejection policy
- **AND** it MUST NOT emit malformed or unbounded JSON

### Requirement: Deterministic root identity and canonical order

r[evaluation_streaming.deterministic_identity_and_order] Mantle MUST assign deterministic root identities and source-order sequence values before parallel dispatch. Live completion order MUST NOT change canonical summary order or stream identity.

#### Scenario: Workers finish in different orders

- **GIVEN** equal admitted roots, source identity, evaluator cohort, selector, and policy under two worker schedules
- **WHEN** workers complete in different orders
- **THEN** live `root-terminal` records MAY have different arrival order
- **AND** both runs MUST produce the same canonical root order and identity inputs

#### Scenario: Root identity input changes

- **GIVEN** a run whose source identity, evaluator cohort, selector, or root identity differs
- **WHEN** Mantle computes the domain-separated BLAKE3 stream identity
- **THEN** the resulting identity MUST differ from the original run identity

### Requirement: Honest partial-run disposition

r[evaluation_streaming.partial_run_disposition] Mantle MUST classify a complete run as `success`, `partial`, `failed`, or `cancelled` from its admitted root outcomes. `partial`, `failed`, and `cancelled` MUST NOT return a successful process disposition.

#### Scenario: Successful and failed roots coexist

- **GIVEN** a complete selected-root set with at least one successful outcome and at least one failed outcome
- **WHEN** Mantle constructs the terminal summary
- **THEN** the summary MUST report `partial`
- **AND** successful root results MUST remain available without reporting full success

#### Scenario: Summary omits a root

- **GIVEN** the selected-root count differs from the terminal outcome count
- **WHEN** summary validation runs
- **THEN** validation MUST fail with a stable incomplete-summary diagnostic
- **AND** the run MUST NOT report `success`

### Requirement: Cancellation and output failure stay terminal

r[evaluation_streaming.cancellation_and_output] Cancellation and machine-output failure MUST stop successful completion. Mantle MUST classify every remaining selected root and MUST NOT accept a late worker success as a replacement for cancellation.

#### Scenario: Operator cancels evaluation

- **GIVEN** evaluation has completed some roots while other selected roots remain active or undispatched
- **WHEN** the operator requests cancellation
- **THEN** Mantle MUST stop new dispatch and request owned worker cancellation
- **AND** every remaining root MUST receive a stable cancelled or not-started outcome

#### Scenario: Machine consumer closes stdout

- **GIVEN** stream mode is active and the consumer closes the output stream before the terminal summary
- **WHEN** Mantle attempts the next record write or flush
- **THEN** the shell MUST return a stable output failure and apply its cancellation policy
- **AND** it MUST NOT report a complete successful stream

### Requirement: Pure outcome core and thin stream shell

r[evaluation_streaming.core_shell_boundary] Root-set admission, transition validation, failure-scope classification, canonical ordering, disposition, and summary construction MUST run in pure deterministic cores over supplied facts. Evaluation, channels, cancellation, encoding, output, and process status MUST remain in shells.

#### Scenario: Equal facts are replayed

- **GIVEN** byte-equivalent admitted roots and outcome observations
- **WHEN** the pure core constructs the summary twice
- **THEN** both summaries MUST be equal
- **AND** the core MUST perform no filesystem, process, network, environment, clock, or output effect

#### Scenario: Shell reports success without an admitted summary

- **GIVEN** the shell has emitted some records but the pure core rejects terminal summary construction
- **WHEN** the command selects its process disposition
- **THEN** it MUST return failure
- **AND** record emission MUST NOT replace summary admission

### Requirement: Reference and compatibility boundary

r[evaluation_streaming.reference_boundary] Mantle MUST adapt root isolation and record streaming without importing `nix-eval-jobs` code, Nix private libraries, Hydra traversal rules, arbitrary evaluator callbacks, or Nix attribute recursion semantics.

#### Scenario: Product source imports the reference implementation

- **GIVEN** a Mantle manifest, source file, protocol, generated schema, or runtime path imports `nix-eval-jobs` code or Nix private evaluator libraries
- **WHEN** the dependency and compatibility gate runs
- **THEN** the gate MUST fail and identify the forbidden edge

#### Scenario: Existing aggregate consumer migrates

- **GIVEN** an existing consumer uses the supported aggregate output during the documented migration period
- **WHEN** the stream surface is introduced
- **THEN** aggregate behavior MUST remain available under its declared compatibility status
- **AND** the new stream MUST use an explicit schema version and selection option

### Requirement: Evaluation stream validation

r[evaluation_streaming.validation] The maintained suite MUST include positive and negative core, pipeline, CLI, schema, ordering, cancellation, worker-loss, and compatibility fixtures. Negative fixtures MUST fail at their target boundary.

#### Scenario: Supported fixture matrix passes

- **GIVEN** all-success, mixed-result, order-variation, cancellation, cache-fact, and replay fixtures
- **WHEN** focused validation runs
- **THEN** each fixture MUST produce its expected root outcomes and terminal disposition
- **AND** equivalent canonical facts MUST produce equal summaries

#### Scenario: Negative fixture fails elsewhere

- **GIVEN** a malformed-stream fixture that fails because an unrelated evaluator or store setup is broken
- **WHEN** validation classifies the result
- **THEN** the fixture MUST NOT count as stream-boundary rejection evidence
