# Frontend-neutral kernel-bundle OCI projection

Mantle implements `mantle-oci-projection-v1` as a generic, local OCI
image-layout projection. OnixOS is the first frontend, but the core does not
interpret kernel releases, ModulePack/BPF Pack compatibility, KBI validity, or
boot policy. Those remain frontend responsibilities.

## Admission boundary

`mantle artifact oci-export` accepts a sealed projection plus the exact
frontend specification material named by that projection. Admission completes
before any CAS object bytes are read. A projection must:

- bind a BLAKE3 hash of the exact frontend spec id/version/material;
- include one redaction-safe reduction of a successful existing
  frontend-artifact admission for every exact `mantle://blake3/<hex>` object
  and no extras; export also receives the exact source-admission bundle and
  recomputes each reduction's BLAKE3 from stable path-free fields, explicitly
  excluding its export-only `build_root` from the projection and report;
- retain frontend component/pack/bundle/manifest identities as opaque roles and
  verify any explicit `oci_layer_blob_sha256` expectation against planned bytes;
- select an explicit OCI platform, exact-blob or canonical-archive mode, media
  type, target path, and named bounds;
- use the complete `mantle-canonical-archive-v1` policy;
- be in canonical role/path order and carry its canonical projection BLAKE3;
- assert no hidden fallback and the required evidence non-claims.

The export-only `source-admissions.json` uses
`mantle-oci-projection-admissions-v1`; its `admissions` array contains the exact
`attestation` objects from the referenced frontend-artifact admission sidecars,
strictly ordered by `artifact_ref`. Build roots may be present in that local
input, but they are export-only and omitted from the path-free reduction hash
material. The domain-separated source-attestation BLAKE3 covers every stable
admission field and enters the sealed projection and export/import reports.
The bundle must not carry credentials: BLAKE3 provides deterministic linkage,
not secrecy, signature
verification, validator authenticity, or attestation truth.

The reviewed Onix fixtures in
`tests/fixtures/kernel-bundle-oci/onix-reviewed/` are frontend handoffs. Their
`blake3:` object references are not silently promoted into Mantle CAS
references. A bounded adapter must import each object, retain its Onix
identity, attach the resulting admission, select layer modes, and seal the
Mantle projection. Raw handoffs fail closed.

## Functional core and filesystem shell

`src/oci_projection.rs` and `src/oci_projection/` own deterministic validation,
canonical archive generation, OCI descriptor construction, digest-role
separation, report identities, and import inspection. Their inputs are bytes
and materialized-object facts; they do not discover host state or mutate the
filesystem.

`src/oci_projection_shell.rs` is the thin imperative shell. It performs bounded
reads, materializes exact CAS refs, recomputes each metadata-aware object
identity, writes a sibling staging layout, self-verifies it, syncs it, and
renames it into place. Import reads descriptors before admission, validates all
referenced blobs, then stores exact blob files. If a later store write fails,
any prior content-addressed bytes remain unreferenced because no successful
import report is published.

## Canonical layer profile

`mantle-canonical-archive-v1` fixes:

- lexical UTF-8 path order and duplicate-path rejection;
- relative non-escaping symlinks only;
- regular files, directories, and symlinks only;
- uid/gid/mtime `0`;
- file mode `0644`, directory mode `0755`, symlink mode `0777`;
- GNU tar headers in deterministic mode;
- no compression, or the explicit deterministic gzip-v1 profile;
- bounded depth, entry count, and aggregate bytes.

An `exact_blob` layer has exactly one root regular-file object and uses its
bytes unchanged. A `canonical_archive` layer mounts one or more admitted
objects at declared relative paths and emits the fixed archive profile.
Special files, hard links, absolute/traversing paths, escaping links, collisions,
size disagreements, and duplicate descriptors are rejected.

## Digest roles

| Role | Algorithm | Meaning |
|---|---|---|
| Mantle object ref | BLAKE3 | Existing metadata-aware frontend CAS object |
| Projection identity | domain-separated BLAKE3 | Canonical admitted projection document |
| Expected external layer digest | SHA-256 | Frontend expectation for one identified OCI layer blob |
| Layer/config/manifest descriptor | SHA-256 | Exact OCI blob bytes required by OCI |
| Layer blob identity | BLAKE3 | Exact layer bytes retained beside OCI SHA-256 |
| Layout identity | domain-separated BLAKE3 | Layout/index plus exact referenced blob graph |
| Export/import receipt | domain-separated BLAKE3 | Redaction-safe report material |
| Onix/KBI identities | Opaque frontend/interoperability fields | Preserved, never substituted for Mantle or OCI roles |

The export report carries the complete sealed, path-free projection so import
can recompute its BLAKE3 rather than trusting an unattached digest. Registry
tags, transport paths, output directories, credentials, source build-root
paths, and full host paths are excluded from equality and reports.

## Local commands

```console
mantle --state-dir ./state artifact oci-export \
  --projection projection.json \
  --spec-material kernel-bundles-spec.md \
  --source-admissions source-admissions.json \
  --out ./kernel-bundle.oci

mantle --state-dir ./state artifact oci-import \
  --layout ./kernel-bundle.oci \
  --report-out import-report.json
```

Local export never contacts or publishes to a registry. A Mantle-produced
layout with an exact valid export report imports as `admitted`; that state means
only that the Mantle projection and exact canonical bytes round-tripped. A safe
external OCI/KBI layout imports as `compatibility-only`, keeps its exact OCI
SHA-256 and blob BLAKE3 identities, and returns fresh Mantle object refs. It
does not regain Onix identity without full frontend canonical reconstruction.

## Registry publication, signature trust, and pull

The production registry shell implements a bounded OCI Distribution subset over
the already-verified local layout. Registry trust policy is typed Nickel, while
Rust owns bounded policy normalization, Ed25519 verification, signature-artifact
linkage, and receipt identity:

```console
mantle --json artifact oci-push \
  --layout ./kernel-bundle.oci \
  --registry https://registry.example.test \
  --repository onix/kernel-bundle \
  --reference reviewed \
  --trust-policy ./registry-trust-policy.ncl \
  --signing-key /secure/registry-signing-key \
  --bearer-token-file ./registry-token \
  --receipt-out push-report.json

mantle --json --state-dir ./fresh-state artifact oci-pull \
  --registry https://registry.example.test \
  --repository onix/kernel-bundle \
  --reference reviewed \
  --expected-manifest-digest sha256:<from-push-report> \
  --expected-metadata-manifest-digest sha256:<from-push-report> \
  --expected-signature-manifest-digest sha256:<from-push-report> \
  --trust-policy ./registry-trust-policy.ncl \
  --bearer-token-file ./registry-token \
  --out ./pulled-kernel-bundle.oci \
  --report-out import-report.json \
  --receipt-out pull-report.json
```

Push uploads or reuses exact descriptor blobs. `<reference>.mantle-metadata`
retains exact `oci-layout`, `index.json`, and export-report bytes. A third
`<reference>.mantle-signature` OCI artifact contains a deterministic detached-
signature document over the exact image-manifest and metadata-manifest SHA-256
pair plus the policy trust domain. Metadata and signature companions are
published before the user-facing image tag; all three are re-read by immutable
digest before success.

The policy authorizes an exact repository set, trusted public keys, required
signer names, minimum distinct-key threshold, and revoked full-key BLAKE3
identities. Signer names are labels: verification tries every matching trusted
key and counts only distinct verified full-key identities. Registry URL,
repository/tag routing, credentials, key material, and credential/key/policy
paths remain outside signed identity. Repository authorization is a separate
verifier-local policy decision, permitting byte-identical mirroring only when
the destination repository is explicitly authorized.

Pull requires expected SHA-256 values for all three manifests. It verifies tag
resolution, signature artifact subject/metadata/domain linkage, the signature
document descriptor, required signers, threshold, revocations, and detached
Ed25519 signatures after downloading only the signature document. Image and
metadata content blobs are downloaded only after that trust decision succeeds.
Exact reconstruction then invokes the ordinary local importer and succeeds only
with `state = "admitted"`. Redirects and ambient proxies are disabled; HTTP
requires explicit `--allow-http` for controlled local registries.

The contracted `mantle-oci-registry-push-report-v2` and
`mantle-oci-registry-pull-report-v2` receipts bind mutable routing names to three
OCI SHA-256 manifests, trust domain, policy BLAKE3, verified signer names,
verified public-key BLAKE3 identities, Mantle layout/projection identities,
transfer/reuse accounting, credential mode, and local import receipt identity.

Successful signature verification authenticates only the immutable digest pair
under the supplied local policy. It does not prove registry authorization,
transparency, revocation freshness, tag immutability, arbitrary registry
compatibility, artifact correctness, kernel/hardware compatibility, module or
BPF safety, bootability, deployability, exactly-once publication, upload
resumption, release eligibility, or authorization to mutate a target. A failed
push may leave unreferenced blobs or companion tags; exact reruns may reuse
verified blobs but are not transactional rollback claims.
