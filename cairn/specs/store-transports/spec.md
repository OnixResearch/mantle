# Store Transports Specification

## Purpose

Defines the `store-transports` capability.

## Requirements

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

Mantle MUST export selected store paths and their recursive closures into deterministic archive streams. Export MUST resolve selectors through local PathInfo, walk declared closure references, preserve signatures and supported CA path-identity metadata, emit records in deterministic order, and fail closed when required closure facts, payload material, or final NAR facts are missing or stale unless an explicit narrower mode is selected and reported.

#### Scenario: Recursive closure export preserves metadata [r[store_transports.archive_export_closure.scenario.recursive]]

- GIVEN a selected output has local PathInfo, references, signatures, supported CA path-identity metadata when present, and artifact attestation sidecars when supported by the implementation
- AND its recorded NAR size and SHA-256 match a fresh render of the final stored node
- WHEN the operator runs recursive archive export
- THEN Mantle MUST include the selected output and its referenced closure members in deterministic order
- AND each record MUST preserve the metadata needed to verify and import the path on another Mantle store.

#### Scenario: Missing closure fact blocks default export [r[store_transports.archive_export_closure.scenario.missing-closure-fact]]

- GIVEN a selected output references a path whose PathInfo, closure metadata, payload, or required attestation material is unavailable locally
- WHEN Mantle plans default recursive archive export
- THEN Mantle MUST fail before writing a successful archive
- AND diagnostics MUST identify the missing path or metadata class.

#### Scenario: Stale final NAR facts block export [r[store_transports.archive_export_closure.scenario.stale-final-nar]]

- GIVEN a selected PathInfo records NAR size or SHA-256 values that do not match a fresh render of its final stored node
- WHEN Mantle plans archive export
- THEN Mantle MUST reject the path before writing archive magic, metadata, or payload bytes
- AND diagnostics MUST distinguish recorded final-NAR facts from the observed final render without replacing or resigning the stale PathInfo.

#### Scenario: Unsigned paths require an explicit escape hatch [r[store_transports.archive_export_closure.scenario.unsigned]]

- GIVEN a selected path lacks a PathInfo signature required by the archive export policy
- WHEN Mantle exports without an explicit unsigned trust or migration option
- THEN Mantle MUST skip or reject that path according to the documented export mode
- AND the final report MUST identify unsigned paths without claiming a fully trusted archive.

### Requirement: Mantle imports archives idempotently with bounded memory [r[store_transports.archive_import_idempotent]]

Mantle MUST import store archives with bounded memory and idempotent behavior. Import MUST verify archive metadata, store prefix, payload digests, final NAR facts, exact node identity, PathInfo signatures, CA-derived path identity, and requested trust policy before persisting state. Import MUST NOT require a marker-normalized CA hash to equal the final NAR hash. If an acceptable path already exists locally, import MUST skip persistence and payload ingestion while draining or seeking over payload bytes without buffering the full path.

#### Scenario: Missing path imports after independent identity checks [r[store_transports.archive_import_idempotent.scenario.import-missing]]

- GIVEN an archive record targets a path absent from the local store
- AND the record has valid metadata, payload BLAKE3, final NAR SHA-256 and size, exact node identity, store prefix, signatures, and supported CA metadata deriving the declared store path
- WHEN Mantle imports the archive under a trust policy that accepts the record
- THEN Mantle MUST ingest the payload without conflating CA path identity with final NAR identity
- AND it MUST persist signed PathInfo and supported sidecars and materialize output content according to the command mode
- AND the import report MUST identify the path as imported.

#### Scenario: Marker-normalized CA differs from final NAR [r[store_transports.archive_import_idempotent.scenario.marker-ca-final-nar]]

- GIVEN a valid CA output used a marker-normalized NAR hash to derive its store path and final marker rewriting produced a different final NAR hash
- AND the CA field derives the signed logical store path while the recorded final NAR facts match the payload and exact node
- WHEN Mantle imports that archive record
- THEN Mantle MUST accept the distinct identities under the declared trust policy
- AND it MUST preserve the CA path-identity metadata without claiming that it hashes the final NAR bytes.

#### Scenario: Existing path is skipped cheaply [r[store_transports.archive_import_idempotent.scenario.skip-existing]]

- GIVEN the local store already has acceptable PathInfo and content for an archive record
- WHEN Mantle imports that record
- THEN Mantle MUST skip persistence and content reingest for that path
- AND it MUST NOT require memory proportional to the skipped payload size.

#### Scenario: Tampered record is rejected [r[store_transports.archive_import_idempotent.scenario.reject-tamper]]

- GIVEN an archive record has a mismatched payload hash, final NAR hash or size, invalid PathInfo signature, CA field that does not derive the declared store path, wrong store prefix, unsupported mandatory metadata, truncated payload, or payload bytes that ingest to a different node than declared
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

### Requirement: Castore transfer sessions are resumable from verified receiver state [r[store_transports.resumable_castore_sessions]]

Mantle MUST support versioned transfer sessions whose canonical manifests bind the requested content identities, artifact classes, logical store prefix, required metadata, named limits, and BLAKE3 manifest identity. Resume planning MUST validate the checkpoint and recompute remaining demand from verified receiver state rather than trusting a cursor alone.

#### Scenario: Interrupted transfer resumes missing content only

- GIVEN a transfer session committed and acknowledged a subset of demanded castore objects before interruption
- WHEN the receiver validates a matching checkpoint and reprobes local object completeness
- THEN Mantle MUST derive the remaining demand deterministically and request only missing or incomplete content
- AND already verified complete objects MUST NOT be retransmitted merely because transport reconnected.

#### Scenario: Stale or tampered checkpoint fails closed

- GIVEN a checkpoint has the wrong manifest digest, policy digest, transfer identity, regressed acknowledgement state, impossible byte counters, malformed bounds, or missing receiver objects
- WHEN Mantle plans resume
- THEN it MUST reject or safely recompute from verified receiver state with a stable reason code
- AND it MUST NOT skip required bytes or declare transfer complete from the checkpoint alone.

### Requirement: Streaming transfer is receiver-driven and backpressured [r[store_transports.receiver_driven_backpressure]]

Mantle MUST separate bounded control messages from bounded artifact chunks and MUST let the receiver grant explicit byte/chunk credit before payload transmission. Named policy MUST bound chunk size, in-flight credit, buffered chunks, object count, total bytes, checkpoint size, and progress limits, and rejection MUST occur before allocating or buffering disallowed payload.

#### Scenario: Credit bounds in-flight data

- GIVEN a sender has more demanded content than the receiver's current credit
- WHEN the sender streams chunks
- THEN it MUST stop transmitting payload when credit is exhausted until acknowledgement grants more credit
- AND receiver memory or disk staging MUST remain within configured named bounds.

#### Scenario: Quota overflow is rejected before persistence

- GIVEN a chunk, object, manifest, checkpoint, or cumulative transfer would exceed configured policy
- WHEN the receiver validates the next operation
- THEN Mantle MUST reject it before disallowed allocation or persistence
- AND it MUST preserve a deterministic transfer-phase diagnostic without importing partial output as successful.

### Requirement: Verified content presence permits early transfer cutoff [r[store_transports.content_presence_early_cutoff]]

Mantle SHOULD stop transfer work when verified receiver state already contains the complete requested content identity and required closure metadata, or when every demanded object has been acknowledged. An expected content-addressed path, sender assertion, partial digest prefix, incomplete directory tree, or unadmitted PathInfo MUST NOT trigger successful cutoff.

#### Scenario: Complete receiver content avoids payload transfer

- GIVEN the receiver verifies the requested root identity, complete object closure, and required metadata before granting payload credit
- WHEN transfer demand is planned
- THEN Mantle SHOULD return an `already-present` completion disposition without requesting artifact payload bytes
- AND later output reuse or import MUST still pass its ordinary trust and admission policy.

#### Scenario: Partial content cannot fabricate completion

- GIVEN the receiver has only a root node, partial directory closure, stale PathInfo, mismatched attestation, or unverified sender claim
- WHEN early-cutoff eligibility is evaluated
- THEN Mantle MUST continue missing-content negotiation or reject the transfer
- AND it MUST NOT report `already-present` or successful output admission.

### Requirement: Mantle explicitly migrates stale signed final-NAR metadata [r[store_transports.pathinfo_final_nar_migration]]

Mantle MUST provide an explicit dry-run-first migration for a single exact signed PathInfo whose recorded final NAR size or SHA-256 is stale while its local castore node and CA-derived store-path identity remain valid. Migration MUST independently measure complete local content, preserve store path, node, references, CA metadata, deriver, and output bytes, replace all signatures over stale facts with exactly one selected local signature, preserve existing artifact-attestation claims and graph facts while refreshing its observed content digest, and emit a bounded structured report. It MUST NOT mutate during dry run, generate a signing key during dry run, accept fragments or batch selection, silently repair ordinary archive operations, or claim recovered historical signer authority.

#### Scenario: Dry run reports an exact stale candidate without mutation [r[store_transports.pathinfo_final_nar_migration.scenario.dry-run]]

- GIVEN one exact full logical store path selects an already-signed PathInfo with complete local content and valid CA-derived path identity
- AND a fresh final NAR render differs from its recorded size or SHA-256
- WHEN the operator runs `store repair-final-nar` without `--execute`
- THEN Mantle MUST report old and observed final-NAR facts, signature count, sidecar disposition, and `would-repair`
- AND it MUST NOT write PathInfo, artifact sidecars, signing keys, or output bytes.

#### Scenario: Execution replaces stale facts and authority [r[store_transports.pathinfo_final_nar_migration.scenario.execute]]

- GIVEN the dry-run candidate remains unchanged under the store mutation lock
- AND the operator supplies `--execute` with an available selected signing key
- WHEN Mantle executes migration
- THEN it MUST persist the freshly measured final NAR size and SHA-256 while preserving every non-signature PathInfo field
- AND it MUST discard all old signatures, add exactly one signature over the repaired Nix fingerprint, refresh an existing artifact attestation without discarding claims or graph facts, verify the persisted result, and report `repaired`.

#### Scenario: Current metadata is an idempotent no-op [r[store_transports.pathinfo_final_nar_migration.scenario.current]]

- GIVEN the exact selected PathInfo already matches a fresh final NAR render
- WHEN the operator runs dry-run or execute mode
- THEN Mantle MUST report `current`
- AND it MUST NOT rewrite PathInfo, signatures, sidecars, signing keys, or output bytes.

#### Scenario: Unsafe candidate fails before mutation [r[store_transports.pathinfo_final_nar_migration.scenario.reject]]

- GIVEN selection is not an exact full logical store path, PathInfo is missing or unsigned, castore content is missing or incomplete, CA metadata is unsupported or does not derive the selected path, an existing artifact attestation is malformed or identifies a different logical path, or staged sidecar preparation fails
- WHEN Mantle plans or executes migration
- THEN it MUST reject with a deterministic reason before PathInfo mutation
- AND it MUST NOT change signatures, metadata, sidecars, content, or output files.

### Requirement: Mantle owns a split nix-archive and castore NAR boundary

r[store_transports.nix_archive_boundary] Mantle MUST pin one reviewed `nix-archive` package version and source identity behind one shared adapter. The adapter MUST own filesystem NAR defaults, case-hack selection, digest behavior, error mapping, and evidence fields. Adapted Snix MUST retain castore NAR rendering and ingest until a separate accepted change proves an equivalent service-backed replacement.

#### Scenario: A caller needs filesystem NAR facts

GIVEN a Mantle caller needs NAR bytes, size, or a recursive hash from a host filesystem path
WHEN the caller uses the shared NAR adapter
THEN the adapter MUST select the reviewed `nix-archive` implementation and explicit Mantle policy
AND the caller MUST NOT select upstream defaults or import `nix_archive::nar` directly.

#### Scenario: A caller needs castore NAR work

GIVEN a Mantle caller starts from a castore node or must ingest a streamed NAR into castore services
WHEN the caller selects its NAR implementation
THEN it MUST use the accepted Snix service-backed path
AND it MUST NOT buffer the complete NAR to force it through a filesystem-only adapter.

#### Scenario: The upstream package changes

GIVEN the package version, Cargo checksum, reviewed source revision, API behavior, or license differs from accepted evidence
WHEN Mantle evaluates the dependency
THEN it MUST treat prior parity and adoption evidence as stale
AND it MUST block the changed dependency until review and positive and negative evidence are current.

### Requirement: Filesystem NAR observations are byte-safe and explicit

r[store_transports.nix_archive_filesystem_observation] Mantle MUST compute selected filesystem NAR observations through the shared adapter. Each observation MUST preserve raw filename and symlink-target bytes, executable mode, canonical child order, explicit case-hack policy, NAR size, and the requested digest. Filesystem I/O MUST run in a thin blocking shell. Observation results MUST NOT grant PathInfo, store, source, or release authority.

#### Scenario: A physical store path is checked

GIVEN a physical output tree and expected final NAR facts
WHEN Mantle checks the tree through the shared adapter
THEN it MUST compare the observed size and digest with the expected facts
AND node identity, signatures, references, content-addressed identity, and persistence MUST remain separate checks.

#### Scenario: A byte-safe tree is observed

GIVEN a valid Unix tree contains non-UTF-8 names, non-UTF-8 symlink targets, executable files, and nested directories
WHEN the adapter encodes or hashes the tree
THEN it MUST preserve those bytes and modes in canonical NAR order
AND it MUST return the same supported NAR facts as the accepted parity corpus.

#### Scenario: The tree changes during observation

GIVEN a path component changes or becomes unreadable during traversal
WHEN the adapter observes the tree
THEN it MUST return a typed non-success result without silent fallback
AND no caller MUST persist PathInfo, lock data, source identity, or a successful receipt from that observation.

### Requirement: nix-archive adoption is parity-gated

r[store_transports.nix_archive_parity] Mantle MUST retain durable parity evidence before each production filesystem NAR seam moves. Evidence MUST compare NAR bytes, size, SHA-256, requested additional hashes, byte-safe names, modes, symlinks, directory order, case-hack behavior, errors, and selected race cases across the applicable implementations.

#### Scenario: The complete parity matrix agrees

GIVEN current upstream identity and all required positive and negative fixtures
WHEN `nix-archive`, Snix, and the available Nix oracle produce the required observations
THEN every required byte and fact MUST agree before the production seam moves
AND the evidence MUST identify unavailable optional oracles without reporting them as passed.

#### Scenario: One required observation differs

GIVEN any required fixture has different bytes, size, digest, mode, name handling, case-hack behavior, or error disposition
WHEN the cutover gate evaluates the evidence
THEN it MUST reject the seam migration with the compared observations
AND production MUST remain on the previously accepted implementation.

#### Scenario: Generated tests exceed a named bound

GIVEN a generated tree would exceed the configured case count, depth, entry count, file-size, or total-byte limit
WHEN the parity harness plans the case
THEN it MUST reject the case before disallowed allocation or filesystem work
AND the rejection MUST NOT count as parity success.

### Requirement: Full-buffer decode and partial restore stay outside production transport

r[store_transports.nix_archive_castore_separation] Mantle MUST NOT use complete-byte-slice NAR decode or non-transactional restore for production archive, cache, remote, or Nario payload paths in this change. A future restore path MUST use named bounds, a fresh staging destination, cleanup, no-replace publication, and post-publication verification.

#### Scenario: A large streamed payload enters Mantle

GIVEN a native archive, Nario archive, HTTP cache, remote build, or shared Rust cache streams a NAR payload
WHEN Mantle validates or ingests that payload
THEN it MUST keep the accepted bounded streaming castore path
AND it MUST NOT collect the complete payload only to call a borrowed-slice decoder or restorer.

#### Scenario: Restore is requested through the new adapter

GIVEN production code requests filesystem restoration through `nix-archive`
WHEN the dependency guard evaluates the request before a separate restore change is accepted
THEN it MUST reject the production use
AND tests or bounded fixture tools MUST remain the only permitted decode or restore users.

#### Scenario: Persistence failure is not reported as success [r[store_transports.pathinfo_final_nar_migration.scenario.persistence-failure]]

- GIVEN all preflight checks pass but PathInfo persistence, sidecar publication, rollback, or post-write verification fails
- WHEN Mantle executes migration
- THEN it MUST return a non-success result naming the failed phase and whether rollback restored the prior PathInfo
- AND it MUST NOT claim repaired, recovered historical authority, archive compatibility, content correctness, or release eligibility.

### Requirement: Nario v2 read compatibility is version-bound

r[store_transports.nario_v2_read_compatibility] Mantle MUST support bounded list and import for one pinned Determinate Nix Nario v2 format. Reports and receipts MUST bind the exact producer revision, format version, supported direction, archive BLAKE3, and unsupported metadata classes.

#### Scenario: Pinned Nario archive is listed

GIVEN the pinned Determinate Nix producer emits a supported Nario v2 archive
WHEN Mantle lists it with explicit `nario-v2` format selection
THEN Mantle MUST report each supported path and bounded metadata without store mutation
AND the report MUST identify the exact external format authority.

#### Scenario: Nario import is selected explicitly

GIVEN a supported Nario v2 archive and matching store policy
WHEN the operator selects `nario-v2` store import
THEN Mantle MUST use the Nario compatibility reader rather than the Mantle-native archive reader
AND the Mantle-native archive default MUST remain unchanged.

#### Scenario: Nario export is requested

GIVEN this change supports Nario list and import only
WHEN an operator requests Nario export
THEN Mantle MUST reject the request as unsupported before writing archive bytes
AND it MUST NOT label Mantle-native output as Nario.

### Requirement: Nario v2 admission is bounded and fail-closed

r[store_transports.nario_v2_bounded_admission] Mantle MUST parse Nario v2 with named bounds and deterministic record-state validation. Import MUST validate path identity, logical prefix, references, NAR size and hash, signatures, supported content-addressed metadata, payload structure, and trust policy before PathInfo admission.

#### Scenario: Valid missing path is imported

GIVEN a supported Nario record names a missing path under the configured logical prefix
AND its metadata, NAR payload, signatures, references, CA facts, and trust policy pass
WHEN Mantle imports the record
THEN it MUST ingest and admit the exact path through ordinary castore and PathInfo services
AND the import report MUST identify the path as imported from Nario v2.

#### Scenario: Existing valid path is skipped with bounded memory

GIVEN the local store already contains an acceptable copy of a Nario record
WHEN Mantle reads that record from a seekable or non-seekable stream
THEN it MUST avoid content reingest and PathInfo replacement
AND it MUST seek or drain the payload without memory use proportional to payload size.

#### Scenario: Invalid record fails before admission

GIVEN a Nario archive has malformed order, duplicate paths, unsupported mandatory metadata, wrong prefix, invalid signature, hash mismatch, truncation, or a named limit failure
WHEN Mantle lists or imports the archive
THEN it MUST return deterministic failure before admitting any new path from that archive
AND it MUST NOT emit a successful archive receipt or treat staged bytes as an admitted store hit.

#### Scenario: Complete archive commits as one visible change

GIVEN every Nario record and payload passes complete archive validation
WHEN Mantle commits the staged import
THEN all new PathInfo records MUST become visible through one bounded mutation step
AND a persistence failure MUST restore the prior visible path state or report an explicit rollback failure.

### Requirement: Nario v2 compatibility has durable validation

r[store_transports.nario_v2_validation] Mantle MUST retain positive and negative fixtures produced by the pinned Determinate Nix implementation. Validation MUST cover list, import, existing-path skip, signatures, references, supported CA metadata, malformed input, truncation, conflicts, and limits.

#### Scenario: External fixture corpus passes

GIVEN current fixtures were generated by the pinned producer and retain their source identities
WHEN the production list and import paths process the corpus
THEN supported positive fixtures MUST preserve all claimed fields
AND every negative fixture MUST fail with its expected stable diagnostic.

#### Scenario: Producer or format identity changes

GIVEN a fixture or claimed producer uses another revision, format version, or unsupported feature set
WHEN compatibility evidence is evaluated
THEN Mantle MUST treat the previous compatibility result as stale or out of scope
AND it MUST NOT extend the compatibility claim without new fixtures and review.
