## ADDED Requirements

### Requirement: Remote input sync consumes verified source-bundle state

r[remote_builds.source_bundle_input_sync] Mantle MUST synchronize remote build inputs using declared CAS, PathInfo, and source-input refs, including verified source-bundle or imported-source-state records. The builder MUST request only missing refs, the client MUST upload only requested refs whose bytes match the declared identity, and upload planning MUST enforce privacy and quota policy before any source, store, proof, or secret-descriptor bytes move.

#### Scenario: imported source state satisfies a missing remote input

GIVEN a remote build request declares a source-input ref
AND the client has a verified imported source-bundle record for that source identity even though the logical source path is absent from the physical store
WHEN the builder requests the missing source ref
THEN the client MAY materialize and upload the source from imported source state
AND the builder MUST verify the uploaded digest, source identity, readiness class, and store-prefix binding before sandbox execution.

#### Scenario: stale or unsupported source input is rejected

GIVEN the client has source material with a stale digest, unsupported source kind, wrong store prefix, missing readiness class, or mismatched source identity
WHEN remote input sync evaluates the upload
THEN Mantle MUST reject the upload before sandbox execution
AND the remote build MUST NOT repair the input by fetching from the network or reading ambient package-manager caches.

#### Scenario: upload privacy policy runs before transfer

GIVEN a remote-builder candidate requires missing input upload
WHEN Mantle plans or starts the remote route
THEN the upload plan MUST summarize bounded source, store, proof, and secret-descriptor classes, object counts, and byte counts
AND a disallowed class or quota overflow MUST reject the route before a remote session receives those bytes.
