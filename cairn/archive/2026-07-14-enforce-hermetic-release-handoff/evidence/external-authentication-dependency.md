# External authenticated-handoff dependency

Recorded: 2026-07-14

Requirement: `mantle.release_provenance.cairn_evidence_handoff.cross_repo_dependency`

Cairn's `authenticate-stack-provenance-inputs` change is complete and archived:

- Cairn revision: `f4a1f8df0d430c1b9431358a388ac1d3c1a823ec`
- archive: `cairn/archive/2026-07-14-authenticate-stack-provenance-inputs`
- archive package manifest BLAKE3: `40ea9765488bd02e362f70d5c9c498932544c80f2231a70f9b5c3ef81cd7df83`
- archive mutation receipt BLAKE3: `8a4250a7db47dd4c013467d65e5667af188aa66599a1b89df6788ef067598fa9`
- Mantle-reviewed dependency receipt BLAKE3: `bf33d82555c7bd3afcbc7adac743782327a5d08e536266f0dfda97d6fe342edf`

Mantle pins the reviewed dependency receipt at
`cairn-policy/evidence/cairn-authenticated-inputs-archive-receipt.json`.
Release-handoff descriptors must explicitly reference bytes with that exact
identity and the matching typed archive metadata. The shell measures and bundles
the bytes; the pure core rejects missing, stale, fabricated, or tampered
dependency observations; release verification remeasures the bundle-local copy.

Passing receipts record
`authentication_status: "archive-authentication-prerequisite-bound-v1"`. This
proves the reviewed archived prerequisite and bundle-local linkage are present.
It does not transfer Cairn's authority to Mantle, independently re-run producer
signatures, or prove Cairn lifecycle, source, build, or release correctness.

Focused positive and negative core, shell, bundle-assembly, and release-verify
tests cover the exact dependency and tampered dependency bytes.
