# Kernel Bundle OCI Projection Specification Delta

## ADDED Requirements

### Requirement: OCI projection remains frontend neutral

r[kernel_bundle_oci.projection] Mantle MUST define a versioned generic OCI projection containing frontend spec/attestation identity, ordered object and layer entries, safe Mantle artifact refs, media types, platform data, annotations, archive policy, expected external digests, named bounds, and non-claims without interpreting Onix module, machine, pack, or deployment semantics.

#### Scenario: Admitted Onix projection is represented as data

- GIVEN Onix supplies a valid attested projection with KBI-compatible media types and annotations
- WHEN Mantle parses the projection
- THEN it MUST retain those values as bounded frontend-supplied data
- AND no Onix role, tag, setting, provider, compatibility, or target-selection rule may be hard-coded into Mantle core.

#### Scenario: Projection requests frontend behavior

- GIVEN a projection asks Mantle to evaluate Onix inventory, select packs, resolve kernel compatibility, authorize deployment, or execute a target mutation
- WHEN projection validation runs
- THEN it MUST reject the request as outside the build-tool boundary.

### Requirement: OCI work requires prior artifact admission

r[kernel_bundle_oci.admission] Before reading content for OCI export, Mantle MUST verify the exact source frontend-artifact admission bundle, domain-separated source-attestation BLAKE3 reductions, exact spec id/version/hash, projection BLAKE3, no-hidden-fallback posture, and availability of every referenced `mantle://blake3/...` object. Before promoting an admitted import, Mantle MUST revalidate the embedded sealed projection and its path-free admission reductions against the imported descriptors and bytes. Source build-root values MUST remain outside the sealed projection and reports.

#### Scenario: Complete admission passes

- GIVEN the projection, full source admissions, their path-free reductions, spec identity, object refs, and object BLAKE3 values all agree
- WHEN OCI preflight runs
- THEN Mantle MAY construct the pure layout plan
- AND the resulting plan MUST bind all admitted identities without copying source build roots.

#### Scenario: Admission or object is stale

- GIVEN a source admission is missing/rejected/reordered, its recomputed reduction differs, a spec or projection identity differs, an object is absent, or a referenced BLAKE3 does not match
- WHEN preflight runs
- THEN Mantle MUST fail before archive or layout materialization
- AND it MUST NOT use a host path, Nix command, registry tag, or unverified file as fallback.

### Requirement: OCI directory layers are deterministic and bounded

r[kernel_bundle_oci.layering] Mantle MUST construct directory layers through a versioned canonical archive policy with lexical paths, normalized separators and metadata, explicit symlink handling, rejected escapes/special files, checked sizes, and named entry/depth bounds; any compression profile MUST be deterministic and identity-bound.

#### Scenario: Equivalent trees produce identical layers

- GIVEN two admitted directory objects contain identical paths, types, and bytes but differ in traversal order or ambient ownership/time metadata
- WHEN canonical layer construction runs
- THEN their canonical archive bytes, BLAKE3, and OCI SHA-256 MUST match.

#### Scenario: Tree violates archive policy

- GIVEN a directory contains an escaping path or symlink, unsupported special file, duplicate canonical path, excessive depth/entries/bytes, or arithmetic overflow
- WHEN layer planning or construction runs
- THEN export MUST fail without publishing a partial OCI layout.

### Requirement: OCI and Mantle digest roles remain separate

r[kernel_bundle_oci.digest_roles] Mantle MUST compute BLAKE3 for source objects, canonical archives, projection, layout description, reports, and receipts, MUST compute or verify SHA-256 where OCI/KBI protocols require it, and MUST reject cross-algorithm or cross-role substitution.

#### Scenario: Both digest families verify

- GIVEN exact component bytes have the admitted Mantle BLAKE3 and produce the expected OCI SHA-256 descriptor
- WHEN export or import validation runs
- THEN both identities MUST be retained in their typed roles
- AND changing registry tags or output paths MUST NOT change the Mantle content identity.

#### Scenario: SHA-256 is supplied as Mantle identity

- GIVEN a projection or report places an OCI/KBI SHA-256 value in a Mantle BLAKE3 role or vice versa
- WHEN digest validation runs
- THEN Mantle MUST reject the artifact even when the hex length is otherwise valid.

### Requirement: OCI layout export is atomic and exact

r[kernel_bundle_oci.export] Mantle MUST materialize admitted projections as verified OCI image layouts through a staging root, write content-addressed blobs and canonical OCI JSON, preserve supplied bounded media types and annotations exactly, and publish only after complete verification and atomic rename.

#### Scenario: Local layout export succeeds

- GIVEN an admitted projection and all exact source objects
- WHEN local OCI export runs
- THEN the final layout MUST contain valid `oci-layout`, canonical index/manifest/config documents, exact blobs, and a receipt binding all BLAKE3 and SHA-256 identities
- AND no temporary or host-specific path may enter canonical JSON or the receipt.

#### Scenario: Export target or write fails

- GIVEN the target already exists, temporary-space preflight fails, a blob write/hash differs, or final verification fails
- WHEN export runs
- THEN no successful final layout may be published
- AND failure evidence MUST identify a bounded stage and issue class without leaking content or credentials.

### Requirement: OCI import verifies before CAS admission

r[kernel_bundle_oci.import] Mantle MUST validate the OCI layout, bounded descriptor graph, paths, media/annotation syntax, sizes, SHA-256 blobs, duplicate relationships, and projection linkage before atomically importing exact objects into BLAKE3 CAS and emitting a frontend-neutral reconstruction response; Mantle-profile archives MUST verify canonically, while external archives MUST pass bounded path/type/link safety inspection and retain an exact blob identity distinct from any reconstructed canonical object identity.

#### Scenario: Mantle-produced layout round-trips

- GIVEN a valid Mantle-produced OCI layout is imported
- WHEN descriptor and object verification completes
- THEN reconstructed object BLAKE3 values and projection relationships MUST match the export receipt
- AND the response MUST contain enough verified data for the frontend to reconstruct its canonical manifest.

#### Scenario: External KBI layout has no Onix admission

- GIVEN an external OCI layout has valid KBI-compatible media types, annotations, and SHA-256 blobs but no valid Onix admission/projection
- WHEN Mantle imports it
- THEN Mantle MAY report verified generic OCI content as compatibility-only input
- AND it MUST NOT claim an Onix bundle, kernel compatibility, deployability, or release eligibility.

#### Scenario: Import is malformed or partial

- GIVEN any descriptor, blob, path, archive, size, digest, or linkage check fails
- WHEN import runs
- THEN no referenced successful import may be committed
- AND partial bytes MUST be rolled back or remain unreferenced and ineligible for artifact admission.

### Requirement: OCI reports join the machine-artifact registry

r[kernel_bundle_oci.reports] Stable OCI export and import reports MUST be Rust-owned machine artifacts registered with exact schemas, generated Nickel review contracts, version policy, positive/negative fixtures, BLAKE3 freshness, consumer policy, and explicit non-claims.

#### Scenario: Report shape drifts

- GIVEN a report DTO, schema, generated contract, fixture, or consumer policy changes without regeneration and review
- WHEN the machine-artifact registry rail runs
- THEN it MUST fail before packaging and identify the stale bound artifact.

### Requirement: OCI projection has positive and negative evidence

r[kernel_bundle_oci.verification] The OCI projection lane MUST include deterministic positive and negative pure-core, CLI, CAS, archive, schema/contract, Onix round-trip, and external-import fixtures covering admission, bounds, digest roles, descriptor integrity, atomicity, redaction, and non-claims.

#### Scenario: OCI projection change is ready to archive

- GIVEN maintainers intend to close the OCI projection change
- WHEN closeout validation runs
- THEN focused positive and negative checks, first-party quality checks, dependency audit, documentation, Cairn validation, and proposal/design/tasks gates MUST pass
- AND no registry transport, kernel compatibility, bootability, deployability, or release claim may be inferred from local layout evidence.
