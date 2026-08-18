## Context

Mantle already imports arbitrary local content into a BLAKE3-addressed frontend artifact store and exports spec-admitted artifacts through a generic receipt boundary. The current destination modes do not define OCI image layout, per-layer descriptors, canonical directory archives, or protocol SHA-256 identities.

OnixOS will own `onix-kernel-bundle-v1`, including KBI compatibility rules and full-bundle identity. Mantle should know only enough to materialize a supplied, attested OCI projection safely. Frontend-owned media type and annotation strings remain data.

## Decisions

### 1. Extend the generic artifact boundary

**Choice:** Add `mantle-oci-projection-v1` as a generic plan accepted by `mantle artifact` export/import paths. The projection records frontend spec identity and attestation, ordered object/layer entries, safe Mantle artifact refs, media types, target platform data, annotations, archive policy, expected external digests, and non-claims. It contains no Onix roles, tags, service settings, provider graph, or deployment commands.

**Rationale:** OCI projection is a build/transport primitive. Kernel meaning stays in the frontend validator.

### 2. Require admission before reading bytes

**Choice:** Export preflight accepts the exact source frontend-artifact admission bundle separately from the path-free projection. It recomputes each domain-separated source-attestation BLAKE3 from stable admission fields while explicitly omitting the export-only `build_root`, requires the resulting reductions to match the sealed projection exactly, and then verifies the projection schema/version, projection BLAKE3, exact spec id/version/hash, no-hidden-fallback flag, and every referenced `mantle://blake3/...` object before filesystem materialization. Tests require otherwise-identical admissions with different build roots to produce the same reduction and sealed projection identity. Source build-root values remain export-only and never enter a projection or report, directly or through identity material. Failure stops before CAS reads or layout creation.

**Rationale:** OCI packaging must not become a route around frontend artifact admission.

### 3. Keep planning pure and the shell boring

**Choice:** Pure cores validate the projection, canonicalize ordering, derive layer/archive plans, calculate expected descriptor relationships, classify import results, and construct report DTOs. The imperative shell reads CAS objects, writes temporary archives/layout files, computes hashes, fsyncs/renames the final layout, and imports verified bytes.

**Rationale:** Digest and descriptor logic should be testable with in-memory byte facts and no filesystem mocks.

### 4. Normalize directory layers deterministically

**Choice:** File objects remain exact byte layers. Directory objects become canonical tar layers with lexical entry order, normalized separators, policy-fixed ownership/mode/timestamp metadata, explicit symlink handling, rejected escapes/special files, checked sizes, and named entry/depth bounds. Compression is either absent or a named deterministic profile included in the projection identity.

**Rationale:** Host metadata and traversal order must not change layer bytes or OCI descriptors.

### 5. Separate digest roles

**Choice:** Mantle computes BLAKE3 for every source object, canonical archive, projection, layout description, report, and receipt. It also computes OCI-required SHA-256 for config, layer blobs, and manifests and verifies any supplied KBI-compatible SHA-256 values. Fields are typed by algorithm and role; cross-role substitution fails even when lengths match.

**Rationale:** OCI interoperability does not change Mantle's canonical identity default.

### 6. Preserve frontend media types and annotations exactly

**Choice:** The projection validator admits bounded, safe media type and annotation key/value syntax but does not interpret KBI IDs, ModulePack/BPF Pack bindings, kernel releases, or component semantics. Export preserves supplied KBI-compatible media types and annotations exactly in canonical OCI JSON. Onix's attestation proves their semantic validity.

**Rationale:** This is the key guard against turning Mantle into an Onix module or kernel resolver.

### 7. Export atomically to a local OCI layout

**Choice:** The first supported destination is an OCI image layout directory written under a caller-selected output root. Mantle writes blobs by digest, canonical `index.json`, `oci-layout`, and an export receipt into a sibling staging root, verifies the complete result, then atomically publishes it. Existing targets are rejected unless an explicit safe replacement policy is later added.

**Rationale:** A local layout is portable to standard registry tooling while keeping network credentials and remote mutation out of the first slice.

### 8. Import descriptor-first and CAS-last

**Choice:** Import validates layout version, bounded JSON, media/annotation syntax, descriptor graph, paths, sizes, SHA-256 blobs, duplicate descriptors, and projection linkage before committing any object to Mantle CAS. Layers declaring the Mantle canonical archive profile must reproduce that profile exactly. External OCI/KBI tar layers are instead inspected under bounded path/type/link safety rules, retained with their exact blob identity, and canonicalized into separate reconstructed object identities without pretending the original tar was Mantle-canonical. The shell then computes BLAKE3, imports exact objects atomically, and emits a reconstruction response. Partial imports are rolled back or remain unreferenced and never report success.

**Rationale:** A malformed OCI layout must not gain trusted local artifact refs through incremental best effort.

### 9. Keep external KBI imports compatibility-only

**Choice:** Mantle can verify and return a generic imported OCI projection even when it lacks an Onix manifest, but it marks frontend admission absent. Only Onix can canonicalize that result into `onix-kernel-bundle-v1`; Mantle never emits an Onix full-bundle claim from KBI annotations alone.

**Rationale:** KBI ID excludes operational payloads and cannot stand in for Onix composition integrity.

### 10. Register reports after the registry expansion

**Choice:** Stable `mantle-oci-export-report-v1` and `mantle-oci-import-report-v1` projections are added to the data-driven machine-artifact registry once `expand-machine-artifact-contract-registry` provides the generic rail. Positive and negative fixtures bind schema, DTO, Nickel contract, digest roles, and non-claims.

**Rationale:** This avoids adding a one-off report checker while that active change is establishing the common authority flow.

## Risks / Trade-offs

- Canonical tar policy can differ from another KBI producer's layer bytes. OCI/KBI compatibility is descriptor-level; Mantle round-trip identity is guaranteed only for its declared canonical profile or exact imported blobs.
- Local OCI layout support does not itself publish to a registry. Standard tools can transport it, while a future authenticated registry adapter can be reviewed separately.
- Importing large kernel/module/firmware layers needs strict bounds and temporary-space preflight to avoid partial disk exhaustion.
- `oci-spec` dependency advisories and version constraints remain part of Mantle's existing dependency-audit policy.

## Non-Goals

- Interpreting Onix kernel, pack, role/tag, machine, provider, or deployment semantics in Mantle.
- Registry authentication, tag mutation, signature policy, transparency logs, or measured boot in the first slice.
- Claiming an OCI/KBI-valid image is bootable, safe, deployable, release-ready, or a canonical Onix bundle.
- Replacing Mantle CAS, PathInfo, release artifacts, or BLAKE3 identity with OCI storage and SHA-256.
