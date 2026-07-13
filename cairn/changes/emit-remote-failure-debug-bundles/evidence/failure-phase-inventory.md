# Remote failure phase fact inventory

Task-ID: I1
Covers: operator_diagnostics.remote_failure_debug_bundle

## Question

Which already-authoritative facts are available when each supported remote-build
failure phase is observed, and which shell owns any ephemeral workspace fact?

## Inspected evidence

- `src/main.rs::run_remote_build_dispatches_async`
- `src/main.rs::record_remote_failure_observability`
- `src/remote_build.rs::admit_remote_production_dispatch`
- `src/remote_build.rs::bind_remote_production_dispatch`
- `src/remote_build.rs::prepare_remote_production_input_transfer`
- `src/remote_build.rs::execute_remote_local_build_linux`
- `src/remote_build.rs::admit_fenced_remote_stdio_output`
- `src/remote_build.rs::import_admitted_remote_stdio_outputs`
- `src/remote_build.rs::complete_remote_production_attempt`
- `src/remote_build.rs::append_remote_production_observability_log`
- `src/remote_attempt_log_store.rs`
- `vendor/snix-build/src/buildservice/bwrap.rs::BuildService::do_build`

## Inventory

| Failure phase | Route / assignment | Attempt / fence | Inputs / policy | Immutable log | Transfer / workspace | Admission / cleanup / report |
|---|---|---|---|---|---|---|
| Route and handshake | Selected endpoint id and framed-stdio route class are present in `RemoteClientBuildOptions`; no worker claim is inferred beyond registered classes. | A production attempt is admitted before child launch. Job id, attempt id, and nonzero fence are authoritative coordinator state. | Concrete request contains normalized input refs, source refs, upload byte count, store prefix, executable payload, expected outputs, and typed failure-debug policy. | Failure telemetry is appended through the existing immutable attempt-log store when the attempt is current; stale-fence rejections use the rejected-observability append path. | Transfer is `not-available`; no sandbox exists. Workspace mode is honestly `ephemeral` or `unknown`, never reconstructed. | No output admission occurred. Coordinator metadata bundle publication is diagnostic-only and cannot alter the protocol failure result. |
| Input transfer | Same selected route and registered worker facts remain available. | Current attempt/fence remains bound into the request and transfer state. | Exact declared refs and bounded transfer policy are present; bearer ticket material is not part of `ConcreteBuildRequest`. | Current attempt log scope plus manifest/head digests are available when persistence succeeds; absence is explicit. | Durable checkpoint facts and interruption classification are available. The worker sandbox may not exist yet. | Admission is `not-available`; cleanup is `not-observed`. Transfer interruption remains resumable and is not forced terminal by bundle emission. |
| Queue / assignment | Coordinator registration and selected endpoint are durable. | Assignment nonce is not exported; only job/attempt/fence identities needed for evidence are exported. | Required system, sandbox class, network class, trusted-output key names, and live output claims are coordinator facts. | Same existing immutable log reference; no duplicate log storage is introduced. | Transfer may be pending or complete. No captured payload is claimed. | Queue or assignment rejection creates no output authority. |
| Sandbox execution | Worker-local route class and bounded capability classes are known. | The concrete request carries the coordinator-issued attempt/fence. | Builder payload, declared inputs, sandbox/network policy classes, and disabled-by-default capture policy are present before execution. | Worker capture does not synthesize a log; coordinator bundle reuses its existing immutable log reference. | `BubblewrapBuildService` owns the ephemeral host workspace. On a failed build it moves the workspace into a worker-owned quarantine root before `TempDir` cleanup. The worker shell reads only exact allowlisted sandbox-relative regular files under `scratches/`, ingests them to bundle CAS, then permission-normalizes and removes the quarantine tree. | Execution failure remains the authoritative error even if capture, publication, or cleanup degrades. Worker report text carries only a digest reference or stable redacted capture reason, never a workspace path or captured content. |
| Output transfer | Route and worker remain unchanged. | Attempt/fence is checked on streamed result handling. | Expected outputs and transfer policy are bound to the request. | Attempt log remains external and immutable. | Transfer manifest/checkpoint facts are available; arbitrary output bytes are not copied into the debug manifest. | No output is admitted until fenced validation succeeds. Transfer cutoff records `accepted=false`. |
| Output admission | Selected worker signing-key names and ordinary trust roots are present. | `admit_fenced_remote_stdio_output` preflights the current attempt/fence before import. | Expected output identities and trust policy are available. | Existing immutable attempt-log ref is reused. | Completed transfer facts are available; the original sandbox need not survive. | Rejection records no admitted result. Replay uses a newly reassigned attempt/fence and calls the same fenced admission and import functions. |
| Output import / completion | Route remains diagnostic only. | Completion applies only to the current replay or original attempt. | Store prefix and expected output names are available. | Completion log remains in the existing store. | Imported output report is ordinary store state, not debug-retention state. | Debug GC is constrained to `remote-failure-debug/bundles`; pure retention rejects records marked as ordinary outputs. Reports expose only bundle digest refs, capture/cleanup codes, replay identity, comparison class, counts, and non-claims. |

## Decision

Use coordinator-owned metadata bundles for every observed production failure and
worker-owned capture bundles only when a typed allowlist explicitly enables
payload capture. Reuse immutable attempt-log scope/head/manifest references.
Never delay or replace the original execution/admission result on diagnostic
failure. Treat the bubblewrap host workspace as shell-owned ephemeral state:
retain before cleanup, capture through no-follow bounded reads, CAS-ingest, then
remove or report stable cleanup degradation.

## Owner

Mantle remote coordinator shell owns route, attempt, immutable-log, transfer,
admission, report, replay, and retention facts. The Mantle worker shell plus
`BubblewrapBuildService` owns pre-cleanup workspace retention. Pure eligibility,
identity, capture admission, redaction, comparison, and retention decisions stay
in `crunch-build::distributed::remote_failure_debug`.

## Next action

Validate the focused multiprocess capture/replay rail and record exact commands
in `implementation-validation.md`; only then mark dependent implementation and
verification tasks complete.
