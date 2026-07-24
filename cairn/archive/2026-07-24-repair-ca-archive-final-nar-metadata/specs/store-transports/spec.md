## MODIFIED Requirements

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
