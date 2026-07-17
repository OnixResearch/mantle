## Implementation

- [x] [serial] I1 Implement pure registry target, metadata-manifest, digest/linkage, transfer-accounting, and push/pull receipt logic with positive and negative unit tests. r[kernel_bundle_oci.registry_transport] r[kernel_bundle_oci.registry_admission] r[kernel_bundle_oci.registry_receipts]
- [x] [serial] I2 Implement the thin ureq/filesystem registry shell with HTTPS default, explicit local HTTP, explicit bounded bearer-token files, disabled redirects/proxies, bounded reads, image-tag-last publication, exact pull reconstruction, and ordinary import admission. r[kernel_bundle_oci.registry_transport] r[kernel_bundle_oci.registry_admission]
- [x] [serial] I3 Add `artifact oci-push` and `artifact oci-pull` public CLI actions with JSON/human receipt output and atomic receipt files. r[kernel_bundle_oci.registry_transport] r[kernel_bundle_oci.registry_admission]
- [x] [serial] I4 Register push/pull receipt DTOs, schemas, generated Nickel contracts, positive/negative fixtures, inventory policy, consumers, BLAKE3 freshness, and non-claims. r[kernel_bundle_oci.registry_receipts]
- [x] [serial] I5 Add deterministic in-process registry CLI tests for authenticated push/pull into fresh state, immutable tag drift, tampered metadata/blob content, denied credentials, interrupted image publication, idempotent rerun/reuse, and no-output/no-admission failures. r[kernel_bundle_oci.registry_verification]
- [x] [serial] I6 Add the registry-backed gallery workflow and reconcile catalog, README indexes, OCI/operator/machine-contract docs, validation rails, network requirements, and exact non-claims. r[kernel_bundle_oci.registry_verification]

## Validation and lifecycle

- [x] [serial] V1 Run the existing OCI baseline before core changes and record any pre-existing failures. r[kernel_bundle_oci.registry_verification]
  - Evidence: `evidence/baseline.md` records pueue task `56`; existing OCI CLI/core/shell suites passed, while the machine-contract checker retained three unrelated pre-existing producer-family coverage gaps.
- [x] [serial] V2 Run focused pure-core, shell, public CLI, gallery, inventory, and machine-contract positive/negative checks. r[kernel_bundle_oci.registry_verification] r[kernel_bundle_oci.registry_receipts]
  - Evidence: pueue task `130` passed local OCI, registry, gallery, inventory, workflow, and machine-contract rails; task `142` reran the final registry core/shell/CLI and machine-contract generation/check/self-test plus the typed-contract integration test.
- [x] [serial] V3 Run Rustfmt, `git diff --check`, dependency audit, the first-party quality gate, Tiger Style, and Tracey coverage. r[kernel_bundle_oci.registry_verification]
  - Evidence: pueue task `142` passed final Rustfmt, Tiger Style, the serialized first-party quality gate, configured `cargo-deny`, and diff checks; task `130` reported pre-sync Tracey `145/145`.
- [ ] [serial] V4 Run Cairn validation and proposal/design/tasks gates, sync and inspect accepted requirements, add evidence-backed Tracey references, and archive only after exact evidence is durable. r[kernel_bundle_oci.registry_verification]
