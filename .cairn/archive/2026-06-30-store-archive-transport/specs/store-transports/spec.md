## ADDED Requirements

### Requirement: Mantle defines a versioned streaming store archive format [r[store_transports.streaming_archive_format]]

Mantle MUST define a versioned Mantle-owned store archive format for offline closure transport. The format MUST place archive metadata and each store-path metadata record before that path's payload bytes, MUST bind the logical store prefix, MUST preserve signed PathInfo and content-addressed metadata, MUST use explicit named limits for variable-length fields and chunks, and MUST fail closed on unsupported versions or malformed record order.

#### Scenario: Metadata precedes payload [r[store_transports.streaming_archive_format.scenario.metadata-before-payload]]

- GIVEN a Mantle archive contains a store path with payload bytes
- WHEN an importer reads that path record
- THEN Mantle MUST read bounded metadata containing store path, references, NAR size, hashes, signatures, CA metadata, and node identity before reading the payload
- AND the importer MUST be able to decide whether the local store already has an acceptable copy before buffering or ingesting the payload.

#### Scenario: Unsupported archive version fails closed [r[store_transports.streaming_archive_format.scenario.unsupported-version]]

- GIVEN an archive has an unknown magic value, unsupported format version, unsupported mandatory feature, malformed record order, duplicate store-path record, or variable-length field beyond a named archive limit
- WHEN Mantle parses the archive
- THEN Mantle MUST reject the archive with deterministic diagnostics
- AND it MUST NOT persist PathInfo, write artifact sidecars, or materialize outputs from that archive.

#### Scenario: Store prefix is bound [r[store_transports.streaming_archive_format.scenario.store-prefix-bound]]

- GIVEN an archive declares a logical store prefix different from the importing Mantle store prefix
- WHEN Mantle validates the archive header or a path record
- THEN Mantle MUST reject the mismatched record or archive before import
- AND diagnostics MUST name the expected and declared prefixes without rewriting paths silently.

### Requirement: Mantle exports recursive closures as deterministic archives [r[store_transports.archive_export_closure]]

Mantle MUST export selected store paths and their recursive closures into deterministic archive streams. Export MUST resolve selectors through local PathInfo, walk declared closure references, preserve signatures and CA metadata, emit records in deterministic order, and fail closed when required closure facts or payload material are missing unless an explicit narrower mode is selected and reported.

#### Scenario: Recursive closure export preserves metadata [r[store_transports.archive_export_closure.scenario.recursive]]

- GIVEN a selected output has local PathInfo, references, signatures, CA metadata when present, and artifact attestation sidecars when supported by the implementation
- WHEN the operator runs recursive archive export
- THEN Mantle MUST include the selected output and its referenced closure members in deterministic order
- AND each record MUST preserve the metadata needed to verify and import the path on another Mantle store.

#### Scenario: Missing closure fact blocks default export [r[store_transports.archive_export_closure.scenario.missing-closure-fact]]

- GIVEN a selected output references a path whose PathInfo, closure metadata, payload, or required attestation material is unavailable locally
- WHEN Mantle plans default recursive archive export
- THEN Mantle MUST fail before writing a successful archive
- AND diagnostics MUST identify the missing path or metadata class.

#### Scenario: Unsigned paths require an explicit escape hatch [r[store_transports.archive_export_closure.scenario.unsigned]]

- GIVEN a selected path lacks a PathInfo signature required by the archive export policy
- WHEN Mantle exports without an explicit unsigned trust or migration option
- THEN Mantle MUST skip or reject that path according to the documented export mode
- AND the final report MUST identify unsigned paths without claiming a fully trusted archive.

### Requirement: Mantle imports archives idempotently with bounded memory [r[store_transports.archive_import_idempotent]]

Mantle MUST import store archives with bounded memory and idempotent behavior. Import MUST verify archive metadata, store prefix, payload digests, PathInfo signatures, CA metadata, and requested trust policy before persisting state. If an acceptable path already exists locally, import MUST skip persistence and payload ingestion while draining or seeking over payload bytes without buffering the full path.

#### Scenario: Missing path imports after verification [r[store_transports.archive_import_idempotent.scenario.import-missing]]

- GIVEN an archive record targets a path absent from the local store and the record has valid metadata, payload digest, store prefix, signatures, and supported CA data
- WHEN Mantle imports the archive under a trust policy that accepts the record
- THEN Mantle MUST ingest the payload, verify the resulting content against the record, persist signed PathInfo and supported sidecars, and materialize output content according to the command mode
- AND the import report MUST identify the path as imported.

#### Scenario: Existing path is skipped cheaply [r[store_transports.archive_import_idempotent.scenario.skip-existing]]

- GIVEN the local store already has acceptable PathInfo and content for an archive record
- WHEN Mantle imports that record
- THEN Mantle MUST skip persistence and content reingest for that path
- AND it MUST NOT require memory proportional to the skipped payload size.

#### Scenario: Tampered record is rejected [r[store_transports.archive_import_idempotent.scenario.reject-tamper]]

- GIVEN an archive record has a mismatched payload hash, invalid PathInfo signature, stale CA field, wrong store prefix, unsupported mandatory metadata, truncated payload, or payload bytes that ingest to a different node than declared
- WHEN Mantle imports the archive
- THEN Mantle MUST reject that record before accepting it as a store hit
- AND it MUST NOT persist or export the tampered output.

### Requirement: Mantle lists archive contents without importing [r[store_transports.archive_list_inspection]]

Mantle MUST provide a read-only archive listing operation. Listing MUST validate the archive envelope and per-path metadata enough to report contents, MUST NOT mutate store state, MUST NOT require payload materialization, and MUST keep memory bounded while showing useful path, size, reference, signature, CA, root, and compatibility metadata.

#### Scenario: Archive list shows metadata [r[store_transports.archive_list_inspection.scenario.list]]

- GIVEN a valid Mantle store archive containing multiple path records
- WHEN the operator runs archive list
- THEN Mantle MUST print or emit structured metadata for each path, including store path, NAR size, reference count or references, signature summary, CA marker when present, and root membership when recorded
- AND it MUST NOT persist PathInfo, write sidecars, or materialize outputs.

#### Scenario: Non-seekable list remains bounded [r[store_transports.archive_list_inspection.scenario.non-seekable]]

- GIVEN archive list reads from a non-seekable stream
- WHEN Mantle advances past payload bytes to inspect later records
- THEN Mantle MAY drain payload bytes in bounded chunks
- AND it MUST NOT buffer entire path payloads in memory.

### Requirement: Archive compatibility claims are evidence-gated [r[store_transports.archive_compatibility_claims]]

Mantle MUST NOT claim byte compatibility with Determinate or upstream Nix nario formats unless current fixture-based evidence proves import and export behavior against that exact external format version. Without such evidence, Mantle MUST identify its archive as a Mantle-native format and report Nix nario compatibility as unsupported or unproven.

#### Scenario: Native format makes no unproven Nix claim [r[store_transports.archive_compatibility_claims.scenario.native-only]]

- GIVEN Mantle implements its native store archive format without external nario v2 fixtures
- WHEN CLI help, reports, docs, or tasks describe the feature
- THEN they MUST call it a Mantle-native archive transport
- AND they MUST NOT claim byte-compatible `nix nario` import or export support.

#### Scenario: Compatibility mode requires fixtures [r[store_transports.archive_compatibility_claims.scenario.fixture-gated]]

- GIVEN a future change adds `--format nario-v2` or equivalent compatibility wording
- WHEN that compatibility support is claimed complete
- THEN durable evidence MUST include positive and negative fixtures produced by an implementation of the target nario version
- AND the claim MUST state the exact external format version, supported direction, and unsupported metadata cases.
