## ADDED Requirements

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
