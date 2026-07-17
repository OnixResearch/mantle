## Implementation

- [x] [serial] I1 Audit the accepted transfer requirements, archived completion evidence, current production client/server wiring, existing gallery, and stale operator claims before changing behavior. r[examples.resumable_remote_transfer_workflow]
  - Evidence: `evidence/baseline.md` records the success contract, approach registry, inspected symbols, stale wording, current production test evidence, false-completion cases, and non-claims.
- [x] [serial] I2 Repair repeated-content demand lookup by selecting the canonical artifact-local chunk index before full descriptor validation, with positive and negative pure-core tests. r[examples.resumable_remote_transfer_workflow]
  - Evidence: `crates/crunch-build/src/distributed/remote_transfer.rs::repeated_chunk_digests_are_disambiguated_by_artifact_local_index` reproduces equal digests at different offsets, accepts the correct second descriptor, and rejects a forged offset; pueue task `69` passes the focused core rail and production gallery fixtures.
- [x] [serial] I3 Extend the checked-in remote-build project with a named deterministic repeated-content multi-chunk selector while preserving the existing small default payload. r[examples.resumable_remote_transfer_workflow]
  - Evidence: `examples/projects/remote-build-loopback/mantle-project.ncl` keeps `payload` as default and adds `resumable-payload`; pueue task `40` evaluates the project and exposes the named derivation.
- [x] [serial] I4 Add production-path positive coverage for durable interruption/resume, stable manifest BLAKE3, missing-chunk reuse, byte identity, and one admitted client output. r[examples.resumable_remote_transfer_workflow]
  - Evidence: `tests/remote_transfer_production.rs::gallery_resumable_remote_transfer_resumes_verified_chunks_and_admits_once`; pueue task `69` passes the production fixture after the repeated-content core repair.
- [x] [serial] I5 Add production-path negative coverage proving tampered acknowledged receiver bytes block resume before output admission. r[examples.resumable_remote_transfer_workflow]
  - Evidence: `tests/remote_transfer_production.rs::gallery_resumable_remote_transfer_rejects_tampered_acknowledged_content`; pueue task `69` passes the fail-closed fixture with an empty client store.
- [x] [serial] I6 Reconcile the remote-transfer guide, project runbook, catalog, and README indexes with the implemented production path and exact debug/deployment/admission non-claims. r[examples.resumable_remote_transfer_workflow]
  - Evidence: `docs/remote-transfer.md`, `docs/operator-workflows.md`, `docs/machine-artifact-contracts.md`, the project runbook, catalog, root/examples/project indexes, and ADR 0027 now agree; `tests/examples_inventory.rs::remote_resumable_workflow_catalog_and_docs_name_the_production_boundary` passes in pueue task `94` and rejects removal of the production rail.

## Verification

- [x] [serial] V1 Run `nix develop -c cargo test -p mantle --test remote_transfer_production 'gallery_resumable_remote_transfer_' -- --nocapture --test-threads=1` and the existing `production_stdio_` regression filter. r[examples.resumable_remote_transfer_workflow]
  - Evidence: `evidence/validation.md` records tasks `69`, `94`, and `108`; both gallery fixtures and the full `production_stdio_` filter pass after the repeated-content repair.
- [x] [serial] V2 Run focused gallery evaluation, inventory, and workflow tests plus the remote stdio CLI regression. r[examples.resumable_remote_transfer_workflow]
  - Evidence: `evidence/validation.md` records project evaluation task `40`, exact inventory/doc task `94`, and the successful inventory/workflow/remote-stdio chain in task `108`.
- [x] [serial] V3 Run focused Rustfmt, `git diff --check`, the first-party quality gate, Tiger Style, and Tracey coverage. r[examples.resumable_remote_transfer_workflow]
  - Evidence: `evidence/validation.md` records task `99` passing the canonical first-party quality gate and task `107` passing Tiger Style, `git diff --check`, and accepted-spec Tracey coverage at 145/145.
- [ ] [serial] V4 Run Cairn validation and proposal/design/tasks gates, sync the accepted requirement, archive only after evidence is durable, then append exact post-archive validation output. r[examples.resumable_remote_transfer_workflow]
