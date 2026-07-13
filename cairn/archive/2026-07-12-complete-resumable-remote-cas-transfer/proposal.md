## Why

The archived `production-remote-build-farm` task list marks streaming resumable transfer and its positive/negative verification complete, while `evidence/2026-07-05-session-4.md` explicitly says that work was not started. The current implementation still carries whole `Vec<u8>` NAR/input payloads and rejects outputs above the inline NAR limit even though `RemoteTransferMode::Streaming` exists.

Mantle needs to repair that status contradiction and implement the accepted production-transfer requirement on top of its existing castore and delta foundations. It should adapt Rio's resumable missing-chunk negotiation without copying `rio-store` or introducing a second CAS.

## What Changes

- Add a versioned canonical transfer manifest over Mantle castore objects, NAR streams, source bundles, PathInfo, attestations, and delta artifacts.
- Negotiate receiver-missing objects and transfer only demanded content through bounded chunks.
- Scope transfer sessions and resume checkpoints to the current remote job, attempt, fence, manifest, and policy.
- Add receiver-issued credits, bounded buffering, upload/download quotas, acknowledgements, and safe reconnect/resume.
- Stop transfer early when verified receiver content already satisfies the requested content identity or when all demanded objects are acknowledged.
- Preserve Nix-compatible NAR SHA-256 verification only where interoperability requires it; use BLAKE3 for Mantle-owned chunk, manifest, checkpoint, and receipt identities.
- Keep full-NAR and delta fallback claim-safe and retain inline payloads only for explicitly bounded fixture/bootstrap paths.
- Add superseding evidence that names the archived contradiction instead of rewriting historical evidence.

## Impact

- **Surfaces**: `src/remote_build.rs`, `crates/crunch-build/src/distributed.rs`, `crates/crunch-store`, `crates/crunch-delta`, remote protocol/config/report schemas, and operator docs.
- **Dependency**: current-attempt resume depends on `fence-durable-remote-attempts`; the transfer core remains usable by non-remote store adapters.
- **Non-claims**: no new CAS, no REAPI compatibility claim, no exactly-once network delivery, no output trust from successful transfer, and no production-streaming completion claim until interruption/resume evidence passes.
- **Validation**: manifest/state property tests, positive interrupted resume, negative stale/tampered checkpoint and quota cases, large-output multi-process transfer, fallback tests, focused Cargo/Kani checks, and Cairn gates.
