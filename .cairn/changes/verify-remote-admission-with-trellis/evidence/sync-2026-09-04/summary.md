# Trellis admission spec sync

Date: 2026-09-04
Implementation commit: `6ff60dcd91437306180819ffc4da27b8b7c1c00f`

The dry run reported `blocked: false`, `changed: true`, and `mutated: false`.

The executed sync reported `blocked: false`, `changed: true`, and `mutated: true`. It added these five requirements:

- `remote_builds.trellis_admission_model`;
- `remote_builds.trellis_admission_safety`;
- `remote_builds.trellis_admission_projection`;
- `remote_builds.trellis_admission_evidence_boundary`;
- `remote_builds.trellis_admission_claim_boundary`.

The accepted remote-build spec changed as follows:

- previous BLAKE3: `16bef677017870273ff7a5034a6b830d5897a408af40572681bc89e163dfd2dd`;
- accepted BLAKE3: `2a2a2e6a752b46411a9aa7c547e0aa1952afc4efb6d83aa5b233babd7d68f629`.

Post-sync Cairn validation, Tracey coverage, and the tasks gate passed. Tracey reported 157 of 157 referenced requirements for the selected profile.
