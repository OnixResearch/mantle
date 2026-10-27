# Resumable remote transfer

Mantle uses one bounded resumable-transfer core for existing castore blobs/directories, NARs, source bundles, PathInfo records, attestations, and delta blobs/chunks. The production local stdio client/server path is wired through `run_stdio_remote_child` and `cmd_remote_serve`; it does not introduce another CAS or replace ordinary output admission.

## Policy

`lib/remote-builders.ncl` exposes a typed `profile.transfer_policy` record. Its defaults bound:

- chunk bytes and total bytes;
- artifact and chunk counts;
- in-flight bytes/chunks and buffered chunks;
- control and checkpoint bytes;
- idle progress and replay rounds.

Rust deserializes this as `crunch_build::distributed::RemoteTransferPolicy` and validates hard limits before manifest acceptance, source reads, payload allocation, receiver persistence, sandbox start, or output import.

## Transient-handle admission

A transient Mantle protocol message MUST NOT introduce an unknown handle,
session, lease, or binding. A lifetime-bearing declaration MUST establish it
first. The receiving building-plane boundary rejects an unknown reference
locally, without adding a rejection frame to the wire. A checkpoint or
acknowledgement records progress, not receiver content or output authority.
See [ADR 0083](../adr/0083-require-declarations-before-transient-remote-handles.md).

| Boundary | Establishing declaration | Established/unknown fixture |
| --- | --- | --- |
| Transfer chunk/session/artifact | Canonical manifest, receiver demand, and chunk credit | `crunch-build` `credit_and_chunk_validation_reject_before_payload_allocation` reserves the exact chunk; an unknown session or artifact fails before payload allocation. |
| Acknowledgement | Manifest-scoped in-flight chunk reservation | `crunch-build` `acknowledgement_is_monotonic_and_digest_checked` accepts matching scope and verified digest; unknown session or digest fails. |
| Resume checkpoint | Canonical manifest and fenced transfer lease; receiver-owned bytes establish content separately | `crunch-build` `receiver_missing_set_is_permutation_invariant_and_resume_requests_only_missing_chunks` resumes verified content; `checkpoint_tamper_scope_regression_missing_object_and_forged_cursor_fail_closed` rejects unknown sessions and acknowledged-but-missing content. |
| Resource-scoped attempt | Coordinator assignment plus matching resource lease | `mantle` `resource_attempt_report_requires_preexisting_matching_lease` accepts the assigned attempt and rejects its report when the lease is missing. |
| Production session binding | Coordinator job assignment naming the current worker, attempt, and fence | `mantle` `production_session_binding_requires_assigned_job_and_worker` resolves the assigned binding and rejects unknown jobs or unassigned workers. |
| Loopback session | Concrete request deriving the client session identity | `mantle` `loopback_session_uploads_missing_input_and_imports_trusted_output` accepts the bound session; `loopback_unknown_session_rejected_before_ticket_redemption` rejects an unrelated session without redeeming the ticket. |
| Output admission | Signed PathInfo and verified content, requested output, prefix, and attestation facts | `mantle` `durable_remote_output_import_persists_signed_pathinfo_attestation_and_report` admits signed content; `builder_rejects_executor_pathinfo_without_signature` rejects an unsigned result, and `durable_remote_output_import_rejects_missing_pathinfo_bundle` rejects missing PathInfo despite transfer progress. |

These checks constrain protocol shape; none substitutes for digest verification,
proves delivery or crash consistency, or promotes an acknowledgement to trust.

## Resume and fencing

A canonical manifest binds the session to the job, attempt, fence generation, policy digest, store prefix, requested content identity, and artifact set. Checkpoints and leases are atomically written under:

```text
<state-dir>/remote-transfers/<session-id>.json
```

A per-session exclusive lock rejects concurrent writers before progress. On reconnect, Mantle probes receiver-owned bytes and recomputes demand. A checkpoint cursor is never proof of content. Wrong-session, wrong-manifest, stale-attempt/fence, expired, regressed, forged, content-missing, or tampered checkpoints fail closed or are invalidated before demand is recomputed. Reassignment invalidates session authority while already verified content remains reusable through ordinary digest probing and GC ownership.

The production client and its child bind an active session to the same assigned job, attempt, and fence within those processes. Durable transfer checkpoints do not make executor effects or session ownership process-durable. Reconnection or reassignment is explicit, not an automatic retry by the typed effect loop; the receiver's verified bytes, not an old acknowledgement, determine what can be reused.

Chunk content identity and chunk occurrence identity are distinct. Equal bytes may produce the same BLAKE3 digest at multiple offsets in one artifact. Mantle selects a demanded occurrence by canonical artifact id plus artifact-local chunk index, then verifies the complete kind/index/offset/size/digest descriptor before granting or consuming credit.

## Production data plane and fallback

The checked local stdio path uses the `no_std + alloc` `crunch-remote-core`
for bounded attempt, transfer, output, receipt, and effect decisions, with
Mantle-owned output facts and selected capability ports in
`crunch-remote-app`. The host adapters still send framed bytes, execute the
builder, check credentials against an observed clock, manage workspace and
attempt persistence, publish telemetry, and perform ordinary physical store
admission. A dependent effect cannot succeed without its matching adapter
observation. This is a scoped I4/I5 composition, not a claim that all remote
resource/locality policy or SSH, local, and external-batch provider behavior
has moved into the core and application ports: those I2/I3 migrations remain
open. Snix `PathInfo` and substitution reports, credential material, handles,
and process/network types stay in std adapters, not the portable contracts.

The portable remote core accepts named policy limits and fixed-width count
facts. Std adapters convert collection lengths with checked `u32`/`u64`
conversions before passing those observations to the core; a count that cannot
be represented is an error, not a truncated claim. This does not turn a
transfer acknowledgement or a worker report into output authority.

Client and worker adapters pass separate `bytes_max` and `time_secs_max`
fields in `crunch_remote_app::TransferEffectLimits` to input and output
transfer effects; neither transfer result grants output-admission authority.
Transfer completion is not store admission: partially published physical
bytes can remain after a failed attempt without becoming an admitted output.

Bounded control DTOs carry manifests, demand, acknowledgement, checkpoint, and completion state. `mantle-remote-transfer-data-frame-v1` carries one credit-reserved chunk over an ordered `Read`/`Write` transport such as stdio or a socket. The receiver validates the bounded header and reserves credit before allocating the payload buffer.

The std transport checks canonical frame, manifest, normalized output-key, and chunk-digest scope against the assigned request; the core's receipt preimage does not verify a signature or confer output trust. Signed PathInfo, verified content, requested output, logical store prefix, and attestation are checked at the ordinary store boundary.

The production local stdio route performs this sequence for both directions:

1. `run_stdio_remote_child` launches the checked production client protocol.
2. `cmd_remote_serve --binding stdio-once --executor local-build` runs the production server path.
3. The client streams only receiver-demanded input artifacts before sandbox execution.
4. The server streams the built output manifest and receiver-demanded chunks.
5. Transfer completion remains non-authoritative until the client admits signed PathInfo, content, requested output, logical store prefix, and artifact-attestation evidence through the ordinary store path.

`RemoteInputUploadArtifact::payload` and `RemoteOutputTransferArtifact::payload` remain compatibility DTOs for bounded fixture/bootstrap and full-artifact seams. Their presence is not streaming evidence. A `streaming` report can be constructed only from a completed runtime transfer report.

The production `--remote-delta` option advertises delta/full/streaming capability. A production delta hit is not yet bound. When delta is unavailable, Mantle records `mode = "full"` with `fallback_reason = "delta-unavailable"`, transfers the full NAR through the same bounded chunk data plane, and still performs ordinary output admission. It does not report a delta hit.

## Completion semantics

- `already-present`: required content, closure metadata, and admitted PathInfo were verified before sender payload work; transferred bytes are zero.
- `demand-satisfied`: every demanded object was receiver-verified and required metadata/admission facts are present.
- `awaiting-admission`: bytes are complete, but trust/admission is not.
- `interrupted`: the fenced checkpoint is durable and reconnect may recompute missing content.

Transfer reports always set `output_admission_claimed` to false. Signed PathInfo, store-prefix, requested-output, artifact-attestation, producer-policy, and claim-strength checks remain separate.

A failed remote execution ends the session with one `BuildExecution` error frame.
The client reports `remote-worker-execution-failed` with the worker's
failure-debug reference, which `mantle remote status` links to the worker's
debug bundle; the executor's own failure text stays on the worker. See the
[remote execution failure release note](release-notes/remote-execution-failure-report-2026-10-25.md).

## Operator and validation workflow

The supported local production workflow is under
[`examples/projects/remote-build-loopback/`](../examples/projects/remote-build-loopback/).
Its default selector remains a small one-use-ticket example; `.#resumable-payload`
produces deterministic repeated-content bytes large enough to cross multiple
production chunks.

The canonical deterministic rail uses a debug-build-only interruption seam:

```bash
nix develop -c cargo test -p mantle --test remote_transfer_production \
  'gallery_resumable_remote_transfer_' -- --nocapture --test-threads=1
```

The positive fixture interrupts after one durable output acknowledgement, starts fresh client/server processes, checks the same manifest BLAKE3, verifies reused bytes and missing-chunk progress, and admits one byte-checked output. The negative fixture changes the acknowledged receiver chunk and proves resume fails with `acknowledged-chunk-missing` before client output admission.

`MANTLE_TEST_REMOTE_INTERRUPT_AFTER_OUTPUT_CHUNKS` and its input counterpart are debug-test seams, not release operator controls. Release binaries ignore them. They provide deterministic cutoff evidence; they do not prove arbitrary process-kill timing or crash consistency beyond the recorded checkpoint boundary.

Focused production regressions also cover multi-chunk input resume, ticket quota rejection before checkpoints/admission, delta-unavailable full fallback, and an 8 MiB output crossing more than one hundred acknowledged chunks.

The focused I4/I5 proofs cover core effect transitions, application ports, production transfer and local stdio adapters, accepted/rejected stdio wire bytes, output-key/fence/replay denial, and a real unskipped worker capture. They do not establish complete I2 resource migration, I3 provider-port cutover, or the Cairn V1–V5 gates. The recorded partial results and the separately repaired live-daemon regression are in the [remote hexagon focused evidence](../.cairn/changes/separate-remote-build-hexagon/evidence/focused-validation.md); the full filtered root suite has not been rerun after that repair.

### Scoped V2 original/current parity

The checked-in V2 parity fixtures include a test-only instrumented `da00f` original signed two-Worker run, not a pristine original executable or whole-CLI successor equivalence. The original used the same job at fences 1→2: a retry before backoff left durable state and bytes unchanged, a stale signed first result was rejected before client PathInfo/export, and the successor signed result was imported and completed. An independent current physical two-Worker test exercises the corresponding fence, backoff, stale-before-import, capacity-release, and signed successor path; the current accepted CLI test exercises one real Worker, not a two-Worker CLI retry.

The accepted current CLI test captures only bounded private Worker **response** stdout. Its parser distinguishes nine semantic control kinds from streaming sideband controls and manifest-bound data headers, then checks each raw output chunk's declared size, artifact/chunk membership, scope, and BLAKE3 digest before advancing. The 4,096 imported bytes, signed PathInfo/NAR, 14 ordered durable telemetry events, and original signed output facts match within that scoped comparison. CLI dispatch matches the Worker's advertised signing-key **name**, as the default local route does; this handshake is not a full-key-material authentication claim. A separate post-import `store verify --trusted-public-keys` with the full public verifying key checks the actual signed store output.

The rejected current CLI case spawns a genuinely absent builder program after dispatch, observes the underlying `ENOENT`, preserves its unused one-use ticket, and records a lost job/failed attempt with six ordered immutable telemetry events and no imported output or PathInfo. The bounded original failed-adapter receipt has the same observed diagnostic class, state, and event projection. Neither original CLI typed EffectIds nor quantified original CLI resources were observed; V2 remains open for the full identities, effects, and whole dual-path compatibility requirements. The exact sources, prior failed focused runs, final 7/7 focused receipt, and nonclaims are in the [focused validation evidence](../.cairn/changes/separate-remote-build-hexagon/evidence/focused-validation.md).

## Gateway build-service evidence

The gateway producer uses the accepted Valence profile
`valence.build-service-evidence.v1`, through
`valence_core::build_service`; it does not define another profile. The host
projects its durable request/attempt/fence, committed transition times and
sequences, public operation-specific Basalt decision, resource-policy replay,
quota entry, measured worker usage and admitted output facts through
`nix_gateway::evidence::GatewayEvidence`. `finish()` runs Valence's validator
before serialization. `Export::publish()` writes an immutable
`<BLAKE3-of-artifact-bytes>.json` below an already-open private directory
capability, using the existing durable-file-publication rail. Export is bounded
to 1 MiB and 64 records per bounded producer category; invalid scope, missing
authority, conflicting reconciliation identity, unsupported role, invalid link,
privacy marker or changed retained artifact bytes is an error, not a partial
success artifact.

Worker inventory and retained output signer identities carry the complete
encoded public key. Nix signature headers carry only its name; that header is
an interoperability check, never authority. Result retrieval matches the full
configured key and verifies its signature, revalidates the current signed
input-addressed derivation and its exact declared outputs, then checks the
retained signed PathInfo against the current measured CAS content. It does
not require an optional action-result cache-discovery index to exist.
Coordinator mutation locks cover bounded state transactions, not async store
or worker execution; committing a result still rechecks the current attempt,
fence and result identity. Temporary private executor diagnostics are not part
of the public JSON/stderr contract.

Reservations and pending reconciliation are real observations. A failed or
partially observed execution can therefore export the `usage` observation
class without claiming a build result. The `build` class appears only with
admitted output identities and a public producer-signature reference. The
initial queued journal entry establishes the transition origin; the exporter
does not invent a prior state or a queued-to-queued execution transition.

Resource selection exports both rollout decisions and canonical replay inputs,
and reruns the selector before labeling that bounded decision `verified`.
For OOM retries the separately replayed escalation edge, not the ordinary
selector's smaller-class decision, records the actual successor class.
The exporter authenticates the retained predecessor usage, requires a positive
dedicated-cgroup OOM delta, checks its exact recorded observation and original
selected-class link, and replays the retained bounded escalation inputs.
Immutable predecessor bundles preserve earlier fences, selections and phases;
each successor records only its own phases. Missing retry-chain links or
unrecognized replay fields fail closed.
Measurements, transitions and usage stay `recorded_only`; output results stay
`signed`; request/attempt/class linkage stays `linked`. A configured signer or
consistent link never establishes output behavior. Exact-child CPU, wall time,
peak RSS and observed stdout/stderr **byte counts** are observations, not log
text or a cgroup tree integral. When present, a dedicated delegated cgroup's
memory peak, measurement-completeness class and signed OOM-kill counter delta
are recorded separately. A sampled workspace allocated-block peak is explicitly
a lower bound with bounded traversal coverage, not scratch high-water authority.
Process-only or unavailable OOM observations cannot produce an escalation edge.
A positive-OOM edge must use the actual signed dedicated-cgroup evidence,
replay the existing escalation planner and link its predecessor and successor
with the checked fence and limits. Backend support is not an observation that
a particular run obtained a delegated cgroup or reproduced OOM; those require
the retained signed runtime records.

The configured gateway's authorized `usage` operation returns bounded attempt
sample counts and measured CPU/output totals, plus **window-wide** project and
account accounting totals. `items` bounds the caller's attempt sample; it does
not truncate the durable accounting totals or their reason counts. Accounting
publishes the unit-schedule version, all three per-unit divisors, window bounds,
limits and reservation/charge totals. Its fixed `project_reason_counts` and
`account_reason_counts` keys are `reserved`, `settled`,
`cancelled_without_final_usage`, `worker_lost`, `partial_observation`,
`untrusted_observation` and `usage_exceeds_reservation`. These are attempt counts,
not monetary prices or measurement-completeness claims. All pending categories
retain reservations. Other windows are excluded; unknown project/account bindings,
inconsistent schedules, unavailable stores and arithmetic overflow fail closed.
Usage requires its own fresh `UsageRead` invocation; result/status permission
does not authorize it. The current production transport is the owner-private
Unix socket and `mantle remote gateway call`, **not an HTTP listener**.

Trusted positive OOM collection is supported only on Linux with a fresh,
worker-owned delegated cgroup-v2 subtree whose CPU and memory controllers are
enabled. `memory.events` must expose an actual positive OOM-kill delta after the
whole attempt subtree is empty. Without delegation, collection is process-only;
other platforms have no positive-OOM collector contract. SIGKILL/exit 137 without
that authenticated counter never authorizes escalation. See the gateway
[configured-host runbook](../.cairn/changes/add-nix-remote-service-gateway/design.md#configured-production-host)
for ticket, holder-invocation, signed store and worker-inventory prerequisites.

Configuring a cgroup does not establish that the executor entered it. The worker
requires a positive acknowledgement of the actual pre-exec migration syscall
within the original wall deadline. Missing, negative or late acknowledgement
keeps the observation process-only and cannot authorize an OOM retry. Reaping
the direct child and reaching pipe EOF do not establish subtree teardown:
dedicated accounting also waits for the kernel's populated flag to clear,
without extending that deadline.

The opt-in host exercise below performs a real 64-MiB cgroup OOM, signs and
authenticates that usage, plans/reserves a linked higher-fence 256-MiB successor,
executes and reconciles that successor, and rejects a separate SIGKILL as OOM.
It requires Python 3, an empty owner-controlled cgroup-v2 subtree with CPU/memory
enabled and swap disabled (`memory.swap.max=0`) for deterministic memory pressure.
It fails rather than capability-skipping if delegation is unavailable. This is
resource-host/collector coverage, **not** full gateway admission/remote dispatch:

```bash
MANTLE_TEST_CGROUP_ROOT="$delegated_empty_root" \
  cargo test -p mantle --bin mantle \
  nix_gateway::resource_host::tests::delegated_oom_signed_successor_and_sigkill_negative \
  -- --ignored --exact --nocapture
cargo test -p mantle --bin mantle \
  nix_gateway::resource_host::tests::public_summary_preserves_pending_reasons_scope_window_and_idempotent_charge \
  -- --exact --nocapture
```

For actual gateway proof, launch the configured local-build worker with
`--usage-cgroup-root "$delegated_empty_root"`, use two reviewed classes on its
same endpoint, submit a signed fixed-input workload that exceeds the first cap
and succeeds within the second, then call status, events, usage and result using
fresh operation-specific holder envelopes. Retain the public request, both
signed usage reports, immutable predecessor publication, escalation replay,
coordinator attempt/fence and lease snapshots, quota ledger, API responses,
final signed output metadata and all source/config/executable BLAKE3 identities.
Repeat with retry disabled, non-opted-in project and explicit SIGKILL: no
successor may appear. Usage bounds 0 and above `MAX_USAGE_ITEMS`, wrong-project
authority and status-only authority must be rejected. Do not relabel the opt-in
host test or a supplied OOM replay flag as this complete gateway execution.

`tests/resource_gateway_runtime.rs` provides this complete private-service path
with fresh scoped development issuer/holder/result keys and normal library UCAN
issuance, FD-delivered `remote ticket create`, signed store publication/import,
the real gateway/worker executables, and actual `gateway binding`/`call` CLI
operations. It creates no coordinator jobs or worker measurements itself. The
gateway configuration explicitly declares per-system `required_resources`; the
fixture's signed input-cache descriptor must supply a real static x86_64 BusyBox
root and its full trusted public key. Ordinary signature and NAR verification
are never bypassed. The positive service case starts at 64 MiB and executes its
successor at 512 MiB; separate services exercise retry-disabled, non-opted-in and
SIGKILL negatives. The process-only case checks retained pending reservations
and rejected usage bounds/project/operation authority.

After integration, execute through the existing guarded workflow, preserving its
admission and accounting controls:

```bash
export MANTLE_GATEWAY_INPUT_PATHS="$signed_input_paths_json"
export MANTLE_GATEWAY_ARTIFACT_ROOT="$fresh_retained_runtime_directory"
export MANTLE_TEST_CGROUP_ROOT="$delegated_empty_root"
cargo test -p mantle --test resource_gateway_runtime \
  -- --ignored --nocapture --test-threads=1
```

The test process and delegated root must belong to the same delegated systemd
user-service subtree, not a root-owned login-session cgroup. An operator can
start a fresh `systemd-run --user --property=Delegate=yes` service, move its shell
to a child control cgroup, enable `+cpu +memory` on the now-empty service root,
then create a separate empty attempt subtree with both controllers enabled and
`memory.swap.max=0`. Run the **same guarded test command** in that service; do
not use service migration to evade its supervisor or resource accounting.
Only Linux/cgroup-v2 x86_64 fixture execution is claimed by this driver. Missing
delegation, missing signed input, failed admission and failed assertions remain
failures, not successful skips.

Each fresh case retains CLI requests/bindings/responses/stdout/stderr/exits,
signed-input import report, exact signed derivation ATerm, coordinator snapshot,
quota state, signed worker reports and immutable evidence files. Start/end BLAKE3
receipts bind the executable and selected production/fixture source bytes and
reject source drift. `private/` also retains **development credentials and holder
envelopes** for the bounded run; keep the whole run owner-private and publish
only redacted public evidence. Do not export that directory as a Valence bundle.
Execution is established only by source-bound retained outcomes, not by these
command examples. The 2026-10-27
[`actual-fenced-oom-successor` receipt](../.cairn/changes/add-evidence-driven-resource-policy/evidence/actual-fenced-oom-successor-2026-10-27.json)
retains Pueue 74's actual gateway rail: **1 passed, 0 failed, 0 ignored**.
The signed OOM predecessor advances to a distinct successful attempt at fence
2; both quota entries settle with predecessor linkage. Retry-disabled,
non-opted-in and SIGKILL services retain fence 1 without a successor. The
owned CPU/memory cgroup subtree finishes empty. Public usage authority/bounds
controls have their separate `public-usage-api-2026-10-27.json` receipt.
These scoped Linux observations do not establish complete VM campaign coverage,
frozen G1/G2 admission, whole-change acceptance, archive or release.

Reuse evidence separates producer result and consumer request, explicit sharing
scope, current `result-read` authority, request/platform compatibility, output
check, producer signature and CAS observation. A submit decision cannot stand
in for consumer reuse authority. Logs, tokens, credentials, private claim sets
and their derived hashes do not belong in the artifact. Valence's exact required
non-claim boundary is retained on every record and the profile: canonical
identity and bounded linkage do not prove correctness, hermeticity, sandboxing,
host isolation, authorization freshness, future resource sufficiency, fair
billing, external status truth or release eligibility.

An authorized consumer alias uses `new_reuse` with the retained producer bundle
selected by its immutable `digest.json` leaf. `read_publication` resolves only
that bounded leaf relative to the private directory capability and verifies its
bytes before parsing the accepted profile. The alias adds a distinct consumer
request, actual current result-read authority and a reuse edge; it preserves the
original producer attempt, result and historical usage records unchanged. It
does not reserve or reconcile execution usage again, invent another fenced
attempt, relabel the producer's output as a consumer build, or infer read/reuse
authority from build-submit permission. Missing current consumer authority,
changed producer artifact, cross-project alias without sharing scope and
producer/account/request/output mismatches all fail closed.

## Claim boundary

This evidence proves bounded local production stdio client/server composition, verified receiver-state resume, repeated-content chunk handling, stable manifest identity, quota/backpressure behavior, and ordinary output admission for the checked fixtures. It does not prove exactly-once network delivery, arbitrary kill-point recovery, a production P2P listener, SSH deployment, independent-machine behavior, hostile-worker honesty, compiler correctness, output trust from transfer alone, release reproducibility, or Kani execution.
