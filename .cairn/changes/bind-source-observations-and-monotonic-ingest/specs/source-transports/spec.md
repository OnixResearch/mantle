# Source Transports Observation and Ingest Delta

## ADDED Requirements

### Requirement: Source observations bind immutable acquisition facts to content

r[source_transports.source_observations.contract] Mantle MUST define a versioned source observation that binds source kind, locator class, immutable revision when applicable, normalized projection, snapshot profile, and measured BLAKE3 content identity.

#### Scenario: Git snapshot produces a stable observation

- **GIVEN** a Git adapter supplies a checked object format, immutable commit, normalized relative projection, snapshot profile, and canonical payload bytes
- **WHEN** Mantle admits and canonicalizes the observation
- **THEN** it MUST produce the same domain-separated observation BLAKE3 for equivalent facts
- **AND** traversal order, checkout path, mirror URL, temporary path, and mutable reference movement MUST NOT change that identity

#### Scenario: Source observation is malformed

- **GIVEN** an observation has an unsupported kind or version, malformed immutable revision, unsafe projection, secret-bearing locator, unsupported profile, malformed BLAKE3, or contradictory fields
- **WHEN** source observation admission runs
- **THEN** Mantle MUST reject it with bounded deterministic diagnostics
- **AND** the invalid observation MUST NOT enter source readiness, release evidence, or durable source state

### Requirement: Locators and mutable references remain observations

r[source_transports.source_observations.locator_boundary] Mantle MUST NOT treat a URL, mirror, local display path, branch, tag, mutable reference, or one inferred genesis commit as source ownership or canonical content identity.

#### Scenario: Mutable reference advances

- **GIVEN** a branch or tag now resolves to a different immutable revision
- **WHEN** Mantle compares it with an existing source observation
- **THEN** the mutable reference MUST NOT preserve the old observation identity
- **AND** only matching immutable revision, projection, profile, and measured content facts MAY satisfy the existing observation

#### Scenario: Mirror location changes

- **GIVEN** two approved mirrors provide the same immutable revision and canonical source payload
- **WHEN** Mantle records acquisition
- **THEN** it MAY record different locator observations
- **AND** the canonical content identity MUST remain the measured BLAKE3 rather than either URL

### Requirement: Source ingest is monotonic and conflict detecting

r[source_transports.monotonic_ingest] Mantle MUST plan source ingest as add, identical reuse, identity conflict, or invalid rejection, and every successful ingest MUST preserve all previously admitted source identities.

#### Scenario: Missing source is added

- **GIVEN** an admitted source observation and payload are absent from durable source state
- **WHEN** the pure ingest planner returns add and the shell commits it
- **THEN** Mantle MUST publish the exact canonical record and payload atomically
- **AND** all previously admitted records, payloads, pins, roots, and readiness facts MUST remain unchanged

#### Scenario: Identical source is imported again

- **GIVEN** durable source state already contains the same canonical source observation and payload
- **WHEN** import repeats with identical bytes
- **THEN** the planner MUST return identical reuse
- **AND** the shell MUST perform no content replacement or duplicate publication

#### Scenario: Existing identity names different content

- **GIVEN** an incoming record reuses an admitted semantic identity with different canonical provenance or payload bytes
- **WHEN** source ingest planning runs
- **THEN** Mantle MUST reject the record as an identity conflict
- **AND** durable records, payloads, pins, roots, and readiness state MUST remain byte-for-byte unchanged

### Requirement: Legacy source records do not gain invented provenance

r[source_transports.source_observations.compatibility] Mantle MUST keep accepted source-bundle v1 records readable and MUST NOT infer unavailable provenance facts from ambiguous legacy metadata.

#### Scenario: Legacy record contains complete unambiguous facts

- **GIVEN** a v1 record contains all required facts for one supported source observation
- **WHEN** the compatibility adapter runs
- **THEN** it MAY produce the checked versioned observation
- **AND** canonical v1 bytes and the original content identity MUST remain unchanged

#### Scenario: Legacy provenance is incomplete

- **GIVEN** a valid v1 content record lacks an immutable revision, projection, profile, or other required observation fact
- **WHEN** the compatibility adapter runs
- **THEN** Mantle MUST preserve it as a legacy content record with a provenance-unavailable disposition
- **AND** it MUST NOT fabricate a stronger source observation or release claim

### Requirement: Source observation claims remain bounded

r[source_transports.source_observations.claim_boundary] Mantle MUST limit source-observation claims to the recorded relation between admitted locator facts, immutable revision facts, projection, snapshot profile, and measured content.

#### Scenario: Source observation verifies

- **GIVEN** a source observation passes admission, rematerialization, content, and compatibility checks
- **WHEN** Mantle reports the result
- **THEN** it MAY claim that the recorded relation matched the supplied bytes
- **AND** it MUST NOT claim source ownership, upstream intent, review quality, license compliance, build correctness, or release eligibility
