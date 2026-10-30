# Source implementation and parent acceptance

Unchecked items are not claimed from code inspection alone.

## Producer source

- [x] [serial] Retain the selected exact upstream source/WIT cohort and generated workspace lock. r[materialize_wrpc_telemetry.producer]
- [x] [serial] Author the actual three-callback emitter modes and real comparison sink. r[materialize_wrpc_telemetry.emitter]
- [x] [serial] Add separate real exporter orchestration, runtime33 profile and member manifest. r[materialize_wrpc_telemetry.producer]
- [x] [serial] Document exact handoff, publication command, real negative inputs and non-claims. r[materialize_wrpc_telemetry.producer]

## Parent execution and publication

- [ ] [serial] Materialize the standalone emitter and inspect actual imports and type-only dependencies. r[materialize_wrpc_telemetry.producer]
  - Blocked: see [integration run](../materialize-wrpc-producer/evidence/integration-2026-10-30.md). Components build; the bundle stops at `wasi-virt-failed` because the `no_std` emitter has no WASI imports.
- [ ] [serial] Run safe span, secret-redaction and 33rd-span queue rejection on the consumer's actual UCAN host. r[materialize_wrpc_telemetry.emitter]
- [ ] [serial] Reject missing, mismatched, substituted, locally built and comparison artifact admission. r[materialize_wrpc_telemetry.producer]
- [ ] [serial] Publish the source revision and record actual immutable output identities. r[materialize_wrpc_telemetry.producer]
