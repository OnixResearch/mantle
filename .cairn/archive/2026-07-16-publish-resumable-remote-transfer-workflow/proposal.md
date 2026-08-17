## Why

Mantle's current production stdio remote path already streams bounded input and output chunks, persists fenced checkpoints, resumes from verified receiver state, and admits outputs through the ordinary signed store path. The accepted specifications and archived evidence describe that implementation, but `docs/remote-transfer.md` and the supported remote-build gallery still say production composition is incomplete. The first gallery-specific multi-chunk fixture also exposed an occurrence-identity bug: two equal-content chunks in one artifact shared a digest, and demand lookup selected the first offset for both. Operators therefore need both an honest workflow and a narrowly repaired repeated-content path.

## What Changes

- Repair demand lookup so repeated equal-content chunks are disambiguated by artifact-local index before the full descriptor is validated.
- Reconcile the remote-transfer documentation with the current production client/server implementation and retain exact transport, admission, debug-seam, and deployment non-claims.
- Extend the supported local remote-build gallery with a deterministic repeated-content multi-chunk payload and an explicit interruption/resume runbook.
- Add production-path positive coverage for durable resume, missing-chunk-only retransmission, stable BLAKE3 manifest identity, and one admitted output.
- Add production-path negative coverage proving tampered acknowledged receiver content blocks resume before output admission.
- Keep the deterministic interruption control debug-only; do not turn a test seam into a production operator control.

## Impact

- **Files**: transfer core and production integration tests, ADR 0027, `docs/remote-transfer.md`, operator/machine-contract docs, `examples/projects/remote-build-loopback/`, examples catalog/index documentation, and lifecycle evidence.
- **Testing**: focused production remote-transfer tests, gallery evaluation/inventory tests, first-party quality/Tiger Style rails, Tracey coverage, and Cairn validation/gates.
- **Non-claims**: no new protocol, no new CAS, no exactly-once network-delivery claim, no production P2P listener claim, no SSH deployment proof, no output trust from transfer alone, and no release-reproducibility claim.
