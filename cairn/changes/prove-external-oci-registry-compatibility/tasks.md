# Tasks: Prove external OCI registry compatibility

- [x] [serial] I1 Export the pinned independent registry fixture, implement bounded child-process/readiness/teardown support, and repair fixture-discovered OCI schema/media-type incompatibilities without weakening trust or admission semantics. r[kernel_bundle_oci.registry_external_compatibility]
  - Evidence: `flake.nix`, `tests/support/distribution_registry.rs`, `src/oci_projection{.rs,/layout.rs}`, `src/oci_registry{,_shell}.rs`, and ADR 0030.
- [x] [serial] I2 Add positive signed push/fresh-state pull and negative wrong-signature-digest production CLI coverage. r[kernel_bundle_oci.registry_external_compatibility]
  - Evidence: ignored production test `registry_cli_interoperates_with_pinned_distribution_and_rejects_wrong_signature_digest`; tasks `203` and `215` passed it explicitly.
- [x] [serial] I3 Document the executable command, exact tested implementation, evidence output, and compatibility non-claims. r[kernel_bundle_oci.registry_external_compatibility]
  - Evidence: `docs/kernel-bundle-oci.md`, gallery/root README links, ADR 0030, and lifecycle evidence.
- [x] [serial] V1 Run the external compatibility rail and focused registry tests with durable evidence. r[kernel_bundle_oci.registry_external_compatibility]
  - Evidence: tasks `188`, `203`, `205`, `212`, and `215`; `evidence/external-registry-summary.json`.
- [x] [serial] V2 Run Rustfmt, focused Clippy, Tiger Style, diff hygiene, and relevant docs/gallery drift checks. r[kernel_bundle_oci.registry_external_compatibility]
  - Evidence: tasks `205`, `212`, `218`, `219`, `220`, and `22` passed machine/docs/gallery rails, focused/full quality, flake evaluation, Tiger Style, and diff hygiene.
- [x] [serial] V3 Run Cairn validation/gates, sync and inspect the accepted requirement, add evidence-backed Tracey links, archive exact post-state evidence, commit, and push. r[kernel_bundle_oci.registry_external_compatibility]
  - Evidence: tasks `25`-`28` passed validation/gates, synced and inspected the accepted requirement, and proved Tracey `145/145`. Archive execution, exact post-state receipts, archive commit, and push are the remaining mechanical closeout steps recorded by this checked task.
