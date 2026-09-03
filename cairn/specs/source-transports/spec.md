# Source Transports Specification

## Purpose

Defines the `source-transports` capability.

## Requirements

### Requirement: Mantle defines a versioned offline source bundle format [r[source_transports.offline_source_bundle_format]]

Mantle MUST define a versioned Mantle-owned source bundle format for portable offline build inputs. The format MUST identify source records by explicit kind and BLAKE3 content refs, preserve declared source identity and downstream use, bind logical store prefix when a record names store paths, enforce named limits for variable-length fields and chunks, and fail closed on unsupported versions or malformed ordering.

#### Scenario: Source record identity is deterministic [r[source_transports.offline_source_bundle_format.scenario.deterministic]]

- GIVEN equivalent source bundle records are emitted in different traversal orders
- WHEN Mantle canonicalizes the source bundle manifest
- THEN Mantle MUST produce the same manifest digest and record order
- AND host temp paths, ambient checkout paths, and serialization map order MUST NOT affect source identity.

#### Scenario: Unsupported bundle shape fails closed [r[source_transports.offline_source_bundle_format.scenario.unsupported]]

- GIVEN a bundle has an unknown magic value, unsupported mandatory feature, duplicate source record, oversized list or metadata field, malformed record order, or store-prefix mismatch for a store-path record
- WHEN Mantle parses or verifies the bundle
- THEN Mantle MUST reject the bundle with deterministic diagnostics
- AND it MUST NOT persist source state or report offline input readiness from that bundle.

### Requirement: Source adapters are language neutral and fail closed [r[source_transports.source_adapter_contract]]

Mantle MUST model package-manager and build-ecosystem inputs through a language-neutral source adapter contract. Each adapter MUST declare lock or manifest identity, source coordinates, expected content refs, offline or network-disable controls, allowed source roots, cache isolation expectations, generated-source boundaries, and unsupported behavior classes. Adapter-specific metadata MAY exist, but generic source-bundle validation MUST NOT require Cargo, Rust, or any single ecosystem's fields for unrelated ecosystems.

#### Scenario: Non-Cargo adapter uses generic source records [r[source_transports.source_adapter_contract.scenario.non-cargo]]

- GIVEN a selected build root uses a non-Cargo package manager or build ecosystem
- WHEN Mantle plans source bundle records for that root
- THEN Mantle MUST represent the source inputs with generic source records plus bounded adapter metadata
- AND missing Cargo-specific fields MUST NOT block the non-Cargo adapter.

#### Scenario: Adapter unsupported behavior fails closed [r[source_transports.source_adapter_contract.scenario.unsupported]]

- GIVEN an adapter cannot prove lock identity, offline controls, cache isolation, allowed source roots, generated-source boundaries, or content refs for a required input
- WHEN Mantle plans or verifies the source bundle
- THEN Mantle MUST emit a deterministic unsupported-adapter diagnostic
- AND it MUST NOT mark the build root offline-ready through that adapter.

### Requirement: Mantle plans and exports deterministic source bundles [r[source_transports.source_bundle_export_plan]]

Mantle MUST plan and export selected build roots' declared source/input closure into deterministic source bundle streams. The plan MUST include supported fixed fetcher inputs, local path sources, VCS checkout snapshots, language package-manager mirrors, bootstrap source archives, provider manifests, toolchain/source-root inputs, proof input blobs, and unsupported or missing source classes without executing the build.

#### Scenario: Plan reports required source inputs [r[source_transports.source_bundle_export_plan.scenario.plan]]

- GIVEN a selected build root depends on declared fetcher inputs, local source trees, VCS snapshots, package-manager mirror material, bootstrap/provider source inputs, or toolchain/source-root inputs
- WHEN an operator requests a source bundle plan
- THEN Mantle MUST report the deterministic set of required source records
- AND the plan MUST identify missing, unsupported, already-local, and exportable records without mutating source or store state.

#### Scenario: Export writes only declared source material [r[source_transports.source_bundle_export_plan.scenario.export]]

- GIVEN a source bundle plan contains supported source records with available payload material
- WHEN Mantle exports the bundle
- THEN Mantle MUST write records and payloads in deterministic order with matching BLAKE3 refs
- AND it MUST NOT read undeclared package-manager caches, sibling checkouts, live VCS remotes, or target/build directories as hidden source inputs.

#### Scenario: Package manager mirrors are language neutral [r[source_transports.source_bundle_export_plan.scenario.language-neutral]]

- GIVEN a selected build root depends on package-manager mirror material such as Cargo vendor roots, npm or pnpm stores, Go module mirrors, Python wheel or sdist directories, Maven repositories, or another modeled language adapter
- WHEN Mantle plans or exports source bundle records for that material
- THEN Mantle MUST represent the records through a generic package-manager mirror source kind plus adapter-specific metadata
- AND Cargo-specific fields MUST NOT be required for non-Cargo package managers.

### Requirement: Source filesystem payloads are canonical and safe [r[source_transports.source_filesystem_canonicalization]]

Mantle MUST canonicalize or reject filesystem source payloads before they can satisfy source-bundle import or offline preflight. The filesystem contract MUST handle relative path normalization, path traversal, absolute paths, symlink targets, modeled executable/readable modes, file kinds, hardlinks, device nodes, FIFOs, sockets, timestamps, Unicode names, and platform-specific case collisions with deterministic diagnostics.

#### Scenario: Safe source tree canonicalizes [r[source_transports.source_filesystem_canonicalization.scenario.safe]]

- GIVEN a source tree contains only supported regular files, directories, safe relative symlinks, and modeled executable bits
- WHEN Mantle canonicalizes the source payload
- THEN Mantle MUST produce deterministic source-record content refs
- AND equivalent trees MUST produce the same refs across machines with the same declared filesystem contract.

#### Scenario: Unsafe source tree is rejected [r[source_transports.source_filesystem_canonicalization.scenario.reject-unsafe]]

- GIVEN a source payload contains path traversal, absolute payload paths, unsafe symlink targets, unsupported device/FIFO/socket files, ambiguous hardlinks, unstable timestamps, invalid names for the target contract, or platform-specific case collisions
- WHEN Mantle imports or verifies the source bundle
- THEN Mantle MUST reject the payload before marking the source ready
- AND diagnostics MUST identify the unsafe filesystem class.

### Requirement: Mantle imports lists and verifies source bundles without network [r[source_transports.source_bundle_import_verify]]

Mantle MUST import, list, and verify source bundles without network access. Import MUST validate manifest and payload digests, persist supported source records idempotently, skip already-present matching records, and reject stale, tampered, or unsupported records. List and verify MUST inspect metadata without executing builds or fetching missing material.

#### Scenario: Import persists matching missing source [r[source_transports.source_bundle_import_verify.scenario.import]]

- GIVEN a source bundle record is absent from local source state and its payload bytes match the declared BLAKE3 ref and source-kind metadata
- WHEN Mantle imports the bundle
- THEN Mantle MUST persist the source record and payload under Mantle-owned source state
- AND the import report MUST identify the record as imported.

#### Scenario: List and verify do not fetch [r[source_transports.source_bundle_import_verify.scenario.no-network]]

- GIVEN a source bundle or imported source state is missing an optional or required remote-origin record
- WHEN Mantle lists or verifies source-bundle material
- THEN Mantle MUST report the missing or unresolved record
- AND it MUST NOT contact the network, run VCS fetch commands, consult language package-manager caches, or execute a build to repair it.

#### Scenario: Tampered source record is rejected [r[source_transports.source_bundle_import_verify.scenario.reject-tamper]]

- GIVEN a source record has mismatched payload bytes, stale logical identity, wrong source kind, unsupported mandatory metadata, path traversal, or mismatched store prefix
- WHEN Mantle imports or verifies the source bundle
- THEN Mantle MUST reject that record
- AND it MUST NOT mark the corresponding build input ready.

### Requirement: Source bundle imports are atomic and pinned when used [r[source_transports.source_bundle_atomic_pinning]]

Mantle MUST import source bundle records atomically and preserve explicit pin or root state when imported sources are intended to support a planned build. Interrupted imports MUST NOT leave partially verified records that can satisfy offline readiness, and unpinned source records MUST be reported as garbage-collection eligible rather than durable build inputs.

#### Scenario: Interrupted import cannot satisfy readiness [r[source_transports.source_bundle_atomic_pinning.scenario.interrupted]]

- GIVEN source bundle import is interrupted after staging bytes but before verified commit
- WHEN Mantle verifies source state for an offline build
- THEN Mantle MUST treat the interrupted records as absent or quarantined
- AND it MUST NOT mark the corresponding source inputs ready.

#### Scenario: Planned build pins imported source records [r[source_transports.source_bundle_atomic_pinning.scenario.pinned]]

- GIVEN a source bundle import is associated with a selected build plan
- WHEN Mantle commits the verified source records
- THEN Mantle MUST persist pin or root metadata binding the imported source records to that planned use
- AND the offline preflight report MUST distinguish pinned durable records from unpinned GC-eligible records.

### Requirement: Mantle provides offline build preflight from imported source state [r[source_transports.offline_build_preflight]]

Mantle MUST provide an offline build preflight that compares selected build roots against imported source/input state before sandbox execution. The preflight MUST report whether required source records are ready, missing, stale, unsupported, untrusted, or would require network access, and it MUST fail closed before build execution when offline input readiness is incomplete.

#### Scenario: Complete source state permits offline build attempt [r[source_transports.offline_build_preflight.scenario.ready]]

- GIVEN selected build roots have a planned source/input closure
- AND imported source state contains every required record with matching identity, digest, and supported source-kind metadata
- WHEN Mantle runs offline build preflight
- THEN the preflight MAY report the roots ready for an offline build attempt
- AND the report MUST bind the source bundle or source-state digest used for that decision.

#### Scenario: Missing source blocks before execution [r[source_transports.offline_build_preflight.scenario.missing]]

- GIVEN selected build roots require a source record that is missing, stale, unsupported, untrusted, or marked network-required
- WHEN Mantle runs offline build preflight
- THEN Mantle MUST fail before starting sandbox execution or remote dispatch
- AND diagnostics MUST identify the source record and blocker class.

### Requirement: Source bundle evidence stays bounded [r[source_transports.source_bundle_non_claims]]

Mantle MUST keep source bundle evidence bounded to source/input availability and identity. A valid source bundle, import report, list report, or offline preflight MUST NOT by itself claim build success, compiler correctness, output reproducibility, remote builder trust, or substitution validity.

#### Scenario: Source bundle proof does not overclaim [r[source_transports.source_bundle_non_claims.scenario.non-claim]]

- GIVEN Mantle verifies a source bundle or reports offline preflight readiness
- WHEN a task, build report, evidence file, or status reply cites that result
- THEN the claim MUST be limited to declared source/input material being present and identity-matched
- AND it MUST NOT claim successful realization, remote output trust, language-planner completeness, compiler correctness, or release reproducibility without separate evidence.

### Requirement: Source bundles realize declared fetcher inputs offline

r[source_transports.source_bundle_realizes_fetcher_inputs] Mantle MUST allow imported and pinned source-bundle records to satisfy declared fixed-output fetcher inputs without live network access. Source-bundle realization MUST verify source-record identity, BLAKE3 payload digest, expected fixed-output hash and mode, source kind, logical store prefix when present, and VCS revision when applicable before admitting the materialized source input into store or castore state.

#### Scenario: imported fixed URL source becomes build input

GIVEN a selected build root depends on a fixed-output URL source
AND local source state contains a pinned imported source record with matching identity, URL metadata, content digest, and expected fixed-output hash
WHEN Mantle realizes inputs for an offline build
THEN Mantle MAY materialize the source input from source state without contacting the network
AND the dependent build MUST consume the verified local source material.

#### Scenario: imported VCS snapshot must match revision

GIVEN a selected build root depends on a fixed-output VCS snapshot
AND local source state contains a pinned checkout payload for the same source identity
WHEN Mantle verifies source-bundle realization
THEN Mantle MUST verify the requested revision or equivalent immutable identity before admitting the input
AND a wrong, missing, or ambiguous revision MUST fail before sandbox execution.

#### Scenario: stale source state fails before execution

GIVEN imported source state is missing, stale, unpinned, unsupported, untrusted, network-required, wrong-prefix, wrong-kind, or fails the fixed-output hash check
WHEN an offline build attempts to realize that input
THEN Mantle MUST reject the source-bundle realization before sandbox execution
AND diagnostics MUST name the source record and blocker class.

#### Scenario: source-bundle realization remains a bounded claim

GIVEN source-bundle realization admits a source input
WHEN Mantle reports that result
THEN the report MAY claim only that declared source/input material was identity-matched and locally available
AND it MUST NOT claim final build output correctness, output trust, compiler correctness, or reproducibility without separate evidence.

### Requirement: Vendor source manifest contract
r[mantle.source_transports.vendor_source_manifests.contract] Mantle MUST support a vendor source manifest contract that records upstream repo, revision, filter, selected paths, measured identities, local edits, refresh command, and non-claims.

#### Scenario: Fresh vendor manifest passes
r[mantle.source_transports.vendor_source_manifests.fixtures.positive]
- GIVEN a manifest row names upstream identity, revision, selected paths, BLAKE3 identities, declared local edits, and required non-claims
- WHEN vendor validation runs
- THEN validation MUST pass and preserve the vendor row identity.

#### Scenario: Stale vendor manifest fails
r[mantle.source_transports.vendor_source_manifests.fixtures.negative]
- GIVEN a manifest row has stale digest, missing revision metadata, unsafe path, or undeclared local edit
- WHEN vendor validation runs
- THEN validation MUST fail with deterministic diagnostics.

### Requirement: Vendor manifest validation
r[mantle.source_transports.vendor_source_manifests.validation] Mantle MUST validate vendor manifests over parsed rows and measured file identities.

#### Scenario: Validation reports identity only
r[mantle.source_transports.vendor_source_manifests.docs]
- GIVEN a vendor manifest validates
- WHEN the supported claim is rendered
- THEN it MUST state that validation proves vendored-source identity/freshness only and not upstream correctness or license compatibility.

### Requirement: Final validation
r[mantle.source_transports.vendor_source_manifests.final_validation] The change MUST include positive and negative fixtures plus focused validation evidence before archive.

#### Scenario: Fixture suite covers vendor health
r[mantle.source_transports.vendor_source_manifests.final_validation.fixtures]
- GIVEN valid and invalid vendor manifest fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass and invalid fixtures MUST fail closed.

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

#### Scenario: Committed source durability is unknown

- **GIVEN** create-new rename committed and parent synchronization then failed
- **WHEN** Mantle maps the shared publication result
- **THEN** it MUST retain the visible destination and report committed durability unknown
- **AND** it MUST NOT classify the result as rejection, durable success, or authorization to replace prior state

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
