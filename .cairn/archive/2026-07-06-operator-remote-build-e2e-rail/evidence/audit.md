# Audit: Operator Remote-Build E2E Rail Surface

Audited against `[depends:remote_builds.operator_e2e_rail]` requirement clauses and
the change's own `r[remote_builds.operator_e2e_rail_composition_proof]` requirement.

## Source Files Audited

- `src/remote_build.rs` (8,334 lines)
- `tests/remote_stdio_cli.rs` (223 lines)
- `scripts/release-determinism-smoke.rs`
- `scripts/check-real-release-determinism-receipt.rs`
- `scripts/check-release-determinism-quality.sh`

## Composition Phases Coverage

### Route Planning
- ✅ Core route planning in remote_build.rs via `Coordinator`/`SchedulerSession` with
  `RoutePhase` frames, `build-request`, `no-builder-for-request`, `wait-policy` responses.
- ✅ Capability matching: `required_capabilities`, `capability_mismatch` diagnostics.
- ✅ Identical request dedupe: `NormalizedBuildKey` with store-prefix-scrubbed inputs.
- ✅ Conflicting live output claims rejected.
- ✅ Coverage: unit tests in remote_build.rs; no composed multi-process fixture.

### Framed Handshake
- ✅ Full handshake: `Hello` → `HelloAck` → `Auth` → `AuthOk` / `AuthReject` with
  version negotiation (`REMOTE_PROTOCOL_ALPN`, `REMOTE_PROTOCOL_VERSION`),
  endpoint identity validation, capability negotiation, and message limits.
- ✅ Stdio mode: `MAINTAIN_STDIO_PROTOCOL` ensures stdout reserved for frames.
- ✅ Tests: `remote_serve_stdio_once_exchanges_frames_and_redeems_ticket` covers
  full stdio handshake → build → output-transfer → done through CLI.
- ✅ Negative: `remote_serve_stdio_once_rejects_unknown_ticket_without_stdout_frames`.
- ⚠️ **Gap**: No single-fixture composition test that exercises handshake AND
  subsequent phases through one driver.

### Source/Input Sync
- ✅ `missing-inputs` frame, `MissingInputsState`, `input-available`/`input-uploaded` flow.
- ✅ `InputUploadState` with NAR inline frames and bounded `MAX_REMOTE_INLINE_NAR_PAYLOAD_BYTES`.
- ✅ Ticket-based upload quotas: `MAX_REMOTE_UPLOAD_BYTES`, per-ticket limits.
- ✅ `UploadGuard` enforces per-session and per-transfer byte limits.
- ⚠️ **Gap**: Input sync is exercised in individual frame tests but not as a
  composed phase following handshake in a single driver.

### Remote Execution
- ✅ `build-finished` frame, `BuildResultState`, output trust checks.
- ✅ Signing key identity in output frames (`path_info_signing_key_id`).
- ✅ Build time limits enforced per session.
- ✅ Stdio/stderr framing with bounded logs.
- ⚠️ **Gap**: No multi-process fixture proves execution after sync.

### Signed Output Admission
- ✅ Output trust checks per `PathInfo.signatures` and `artifact_attestation_digest_blake3`.
- ✅ `OutputTrustPhase` with `output-trust-ok`/`output-trust-fail` frame.
- ✅ Artifact attestation digest included in `build-finished` frame.
- ⚠️ **Gap**: No fixture proves that a signed output is admitted and an unsigned
  or mismatched output is rejected as part of a composed flow.

### Bounded Observability
- ✅ Log chunk limits (`MAX_REMOTE_LOG_CHUNKS`, `MAX_REMOTE_LOG_BYTES`).
- ✅ Stderr truncation with `<stderr-truncated>` marker.
- ✅ Status frame counts bounded.
- ✅ Diagnostics length bounded (`MAX_REMOTE_STATUS_DIAGNOSTIC_BYTES`).
- ⚠️ **Gap**: Observability bounds not asserted in a composed multi-process proof.

## Negative Cross-Seam Cases

### No Output Trust
- ⚠️ No dedicated test for rejected unsigned outputs in stdio mode.
- ✅ Output trust validation code exists (`TrustPhase`), but not composed.

### Unframed Stdout
- ✅ `tests/remote_stdio_cli.rs` has `unframed_stdout_before_greeting_rejects` and
  `unframed_stdout_during_build_rejects` tests — these are per-seam tests.

### Stale Source State
- ⚠️ Not covered in remote-build e2e tests.
- ✅ Source staleness is part of `crunch-project`'s `list_stale`/`refresh` flow.

### Upload Quota / Privacy Overflow
- ✅ Per-session `UploadGuard` with byte limits.
- ⚠️ Not tested as a negative fixture in multi-process e2e.

### Fallback Without Explicit Policy
- ⚠️ No fallback-policy test in remote-build e2e.

## Evidence Model

- ⚠️ No existing machine-readable evidence record for remote-build rail composition.
- ✅ Session non-claim constant `REMOTE_SESSION_NON_CLAIM` exists.
- ✅ `REUSE_PERCENT` tracking for delta transfer statistics.
- ⚠️ No `rail_version`, `fixture_id`, `composition` phases array.

## Summary

| Phase | Covered in Unit | Covered in Composed Fixture |
|---|---|---|
| Route planning | ✅ | ❌ |
| Framed handshake | ✅ | ⚠️ partial |
| Input sync | ✅ | ❌ |
| Execution | ✅ | ⚠️ partial (stdio test covers build→output) |
| Output admission | ✅ | ❌ trust validation |
| Observability | ✅ | ❌ bounds assertion |
| Negative: no trust | ❌ | ❌ |
| Negative: unframed | ✅ per-seam | ❌ composed |
| Negative: stale source | ❌ | ❌ |
| Negative: quota overflow | ⚠️ code exists | ❌ |
| Negative: fallback | ❌ | ❌ |
| Evidence record | ❌ | ❌ |

**Gaps to address in I2/I3/I4:**
1. Single bounded multi-process fixture for full composition
2. Machine-readable evidence with `rail_version`, `fixture_id`, `composition` phases
3. Composed negative cross-seam cases (no trust, unframed stdout mid-flow, stale
   source, quota overflow, fallback without policy)
4. Redaction assertions on evidence
5. Determinism assertions on repeated runs