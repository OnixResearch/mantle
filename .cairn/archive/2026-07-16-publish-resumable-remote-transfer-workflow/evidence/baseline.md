# Baseline and search record

Recorded: 2026-07-16

## Goal and completion evidence

The exact goal is to publish an honest supported gallery workflow for Mantle's production resumable remote transfer. Completion requires the checked-in project to cross multiple production stdio transfer chunks, interrupt after a durable acknowledgement, resume the same fenced manifest from fresh processes, reuse verified receiver content, admit exactly one byte-verified output through the ordinary store path, and reject tampered acknowledged content before admission. Catalog/docs/lifecycle evidence must match that behavior.

False completion includes a standalone transfer-shell test, a negotiated `streaming` label, a whole inline payload, a docs-only claim, a retry that discards the checkpoint, or a test that never verifies ordinary output admission.

Audit risks are stale archived claims, stale current prose, debug seams promoted as product controls, transfer success promoted into trust, cursor trust, duplicate client output writes, and local stdio evidence generalized to P2P/SSH/multi-host deployment.

Search budget: four mechanism families, repository-owned source/spec/archive evidence only, two focused baseline command rounds, no external network retrieval, and one adversarial production fixture before acceptance. Valid outcomes are validated, blocked with an exact production seam, or exhausted with the strongest checked boundary.

## Inspected implementation and authority

- `cairn/specs/remote-builds/spec.md` requires `r[remote_builds.production_streaming_transfer]`.
- `cairn/specs/store-transports/spec.md` requires verified receiver-state resume and receiver-driven backpressure.
- `cairn/archive/2026-07-12-complete-resumable-remote-cas-transfer/` records the production replacement, fallback, 8 MiB rail, accepted-spec sync repair, and post-archive validation.
- `src/remote_build.rs::run_remote_production_client_protocol` sends production input manifests/chunks and receives production output manifests/chunks.
- `src/remote_build.rs::receive_remote_production_input_transfer` receives streamed input artifacts before sandbox execution.
- `src/remote_build.rs::send_remote_production_output_transfer` sends the built output through receiver demand and bounded chunks.
- `src/remote_build.rs::run_stdio_remote_child` and `cmd_remote_serve` are the production local stdio composition path.
- `src/remote_transfer.rs` persists fenced checkpoints and reprobes receiver bytes before resume.
- `tests/remote_transfer_production.rs` uses the public `mantle build --builder ... --ticket ...` path with a local production `stdio-once` worker.

The contradictory current sources are `docs/remote-transfer.md`, which says the production client/server protocol is not wired, and the `remote-build-loopback` runbook/catalog non-claim, which still excludes resumable large-artifact transfer despite the later production implementation and archived evidence.

## Approach registry

| Family | Mechanism | Result |
|---|---|---|
| protocol-reimplementation | Add new DTOs, transport, or CAS | Falsified: current production code already streams and resumes; duplication would weaken the single identity/admission path. |
| docs-only reconciliation | Correct stale prose | Blocked as sole route: necessary but cannot prove the gallery exercises production bytes. |
| standalone-shell promotion | Cite shell tests as production | Falsified: those tests do not establish client/server composition. |
| production-gallery composition | Extend the checked-in project and run it through the production stdio path with resume/tamper fixtures | Surviving candidate. |

The lenses were performed serially and are therefore correlated. Repository tests and lifecycle validators remain authoritative.

## Baseline commands

Pueue task 29 ran the isolated flaky resource-dispatch baseline and then the production transfer filter serially:

```text
nix develop -c cargo test -p mantle --bin mantle remote_build::tests::concurrent_resource_dispatches_commit_once_and_never_overcommit -- --exact --nocapture --test-threads=1
nix develop -c cargo test -p mantle --test remote_transfer_production 'production_stdio_' -- --nocapture --test-threads=1

running 7 tests
test production_stdio_delta_unavailable_falls_back_to_bounded_full_nar_and_admits ... ok
test production_stdio_drops_malformed_trace_and_survives_otlp_outage ... ok
test production_stdio_exports_prometheus_and_propagates_bounded_trace_context ... ok
test production_stdio_rejects_ticket_upload_quota_before_checkpoint_or_admission ... ok
test production_stdio_resumes_interrupted_multi_chunk_input_upload ... ok
test production_stdio_resumes_missing_chunks_and_imports_output ... ok
test production_stdio_streams_and_admits_8_mib_output ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out
```

An earlier parallel broad `remote_build::tests::` baseline in task 25 reported 135 passes and one failure in the unrelated process-global resource-dispatch concurrency test. The exact serialized rerun passed before the production filter. No transfer implementation change is justified by that unrelated parallel baseline.

## Non-claims

This baseline does not prove exactly-once network delivery, arbitrary process-kill recovery, production P2P listening, SSH deployment, multi-host behavior, hostile-worker honesty, compiler correctness, output trust from transfer alone, release reproducibility, or Kani execution.
