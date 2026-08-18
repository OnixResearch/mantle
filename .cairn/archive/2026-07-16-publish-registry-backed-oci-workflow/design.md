# Design: Registry-backed OCI publication

## Context

The accepted `kernel-bundle-oci` capability already owns deterministic layout construction, OCI SHA-256 versus Mantle BLAKE3 separation, exact export reports, descriptor-first import, and CAS admission. The missing surface is transport. The implementation must not duplicate projection or import semantics inside the network shell.

The bounded target is an OCI Distribution-compatible registry selected explicitly by the operator. Deterministic validation uses an in-process loopback registry implementing only the reviewed endpoint subset; it is not an independent-registry compatibility claim.

## Goals

- Push a previously admitted Mantle OCI layout without external CLIs.
- Pull the exact image and Mantle metadata by two immutable expected digests.
- Reconstruct byte-identical `oci-layout`, `index.json`, descriptor blobs, and Mantle export report.
- Reuse existing local import admission before reporting success.
- Keep credentials, transport paths, and mutable tags out of content identity.
- Produce stable contracted receipts and fail-closed positive/negative evidence.

## Non-goals

- General registry authentication negotiation, OAuth token exchange, mTLS, signing, Notary/Cosign, referrers discovery, deletion, garbage collection, upload resumption, cross-repository mounts, multi-platform indexes, exactly-once publication, or arbitrary registry compatibility.
- Treating registry possession as trust, authorization, frontend admission, release eligibility, kernel compatibility, bootability, or deployability.

## Decisions

### 1. Use a narrow OCI Distribution endpoint subset

**Choice:** The shell uses `GET /v2/`, blob `HEAD`, upload `POST` plus digest-finalizing `PUT`, manifest `PUT`, and manifest/blob `GET`. Redirects and ambient proxies are disabled. HTTPS is mandatory unless `--allow-http` is explicit.

**Rationale:** This is enough for deterministic push/pull while keeping credential authority and network behavior inspectable. External `docker`, `podman`, `oras`, or `skopeo` fallback would hide inputs and receipts.

### 2. Preserve Mantle metadata with a subject-bound companion artifact

**Choice:** Push the ordinary OCI image manifest under the requested tag. Also publish a deterministic OCI artifact manifest under `<tag>.mantle-metadata`; its subject is the exact image-manifest descriptor and its three blobs are the exact `oci-layout`, `index.json`, and `mantle-oci-export-report.json` bytes. Publish this metadata manifest before publishing the image tag, and publish the image tag last.

**Rationale:** A registry stores manifests/blobs, not local image-layout sidecars. The companion artifact preserves exact layout and admission bytes without altering the ordinary image manifest. Subject verification prevents unrelated metadata from regaining admitted status. Publishing the user-facing image tag last prevents a successful-looking tag from appearing before its required Mantle metadata exists. ADR 0028 records the durable companion and dual-digest handoff invariant.

**Alternative rejected:** A local-layout tarball as the only registry artifact would weaken ordinary OCI image interoperability. Embedding the export report into the image config/layers would change the already-proven layout identity. Referrers discovery is outside the bounded compatibility target, so the deterministic companion tag is explicit.

### 3. Require immutable pull expectation

**Choice:** `oci-pull` requires both `--expected-manifest-digest sha256:...` and `--expected-metadata-manifest-digest sha256:...`. It resolves both requested tags, computes each returned document's SHA-256, compares registry headers, and rejects either mismatch before downloading or publishing a layout.

**Rationale:** Tags are mutable routing names, not trusted identities. The image digest alone does not authenticate an unsigned Mantle export report: a registry writer could replace the companion metadata while retaining the image bytes. A push receipt supplies both manifest digests needed for later pull and closes that substitution seam without claiming registry trust or signatures.

### 4. Keep authentication explicit and redacted

**Choice:** Optional bearer credentials come only from `--bearer-token-file`. The shell performs a bounded regular-file read, trims one textual token, rejects control/empty/oversized material, applies it only to the configured authority, and records only `anonymous` or `explicit-bearer-file` in receipts.

**Rationale:** Command-line token values leak through process inspection; ambient credential discovery makes the authority surface unreviewable. The credential path and bytes must not enter receipts or content identity.

### 5. Functional core and imperative shell

**Choice:** `src/oci_registry.rs` owns target validation, companion-reference derivation, metadata manifest construction/verification, digest/linkage checks, transfer accounting, and receipt identity. `src/oci_registry_shell.rs` owns explicit file reads, HTTP calls, bounded response reads, staging writes, and orchestration. `artifact_cmd.rs` only resolves CLI paths and emits reports. Existing `oci_projection` and `oci_projection_shell` remain authoritative for layout and import validation.

**Rationale:** Registry protocol reasoning can be tested without sockets, while the HTTP/filesystem shell remains thin and deterministic.

### 6. Make retries content-addressed, not transactional

**Choice:** Blob existence probes allow a rerun to reuse already-uploaded content after an interrupted attempt. No success receipt is emitted until the companion and image manifests are re-read by immutable digest and verified. Failed attempts may leave unreferenced blobs or a companion tag; they must not leave a final local pull layout/import report or a successful receipt.

**Rationale:** OCI Distribution has no transaction spanning blobs and two manifests. Content-addressed retry is honest and interoperable; exactly-once or resumable-upload claims would be false.

### 7. Contract push/pull receipts

**Choice:** Add separate `mantle-oci-registry-push-report-v1` and `mantle-oci-registry-pull-report-v1` DTOs to the machine-artifact registry with exact JSON Schemas, generated Nickel contracts, positive/negative fixtures, version policy, BLAKE3 freshness, and explicit consumers/non-claims.

**Rationale:** These reports cross operator and automation boundaries and therefore require the same authority flow as local OCI reports.

## Approach registry

| Family | Mechanism | Claim | State | Discriminating evidence |
|---|---|---|---|---|
| External CLI wrapper | Invoke Docker/ORAS/Skopeo | Fast transport integration | Rejected | Loses explicit credential, digest, and receipt authority. |
| Layout archive artifact | Publish one tar/blob | Exact local bytes survive | Rejected | Does not publish the ordinary image manifest as the primary OCI object. |
| Modify image manifest | Add Mantle report as layer/config | Metadata travels with image | Rejected | Changes the proven local layout and frontend descriptor identity. |
| Referrers discovery | Publish subject artifact and discover via referrers API | Standard metadata lookup | Blocked | Referrers support is not a bounded baseline across target registries. |
| Deterministic companion tag | Subject-bound metadata artifact plus image tag last | Exact layout round trip with explicit lookup | Active | In-process registry E2E plus immutable digest/tamper/auth/interruption negatives. |

## Validation budget

- **Source budget:** existing OCI core/shell, CLI, machine-contract registry, accepted spec, archived evidence, and ureq API source.
- **Mechanism budget:** five families above; one surviving implementation route.
- **Round budget:** baseline; pure-core tests; focused shell/CLI tests; adversarial registry tests; gallery/inventory tests; machine-contract rail; first-party quality/Tiger Style/Tracey; lifecycle sync/archive.
- **Allowed terminal outcomes:** validated and archived; exact protocol/tool blocker; exhausted bounded compatibility route; or user decision required for a broader authentication/referrers scope.

## Risks

- Registry implementations may vary in accepted artifact-manifest behavior; validation proves the reviewed Distribution subset only.
- Image and companion tags are mutable; both expected digests plus subject verification are mandatory.
- Authentication failures must not expose response bodies or credentials in diagnostics.
- A partial push can leave unreferenced content; docs and receipts must not call the operation transactional.
- Machine-contract generation can reveal unsupported DTO/schema shapes; simplify receipt fields rather than weakening the checker.
