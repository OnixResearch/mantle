# Source implementation and parent acceptance

Unchecked items are not claimed from code inspection alone.

## Producer source

- [x] [serial] Retain the selected exact upstream source/WIT cohort and generated workspace lock. r[materialize_wrpc_telemetry.producer]
- [x] [serial] Author the actual three-callback emitter modes and real comparison sink. r[materialize_wrpc_telemetry.emitter]
- [x] [serial] Add separate real exporter orchestration, runtime33 profile and member manifest. r[materialize_wrpc_telemetry.producer]
- [x] [serial] Document exact handoff, publication command, real negative inputs and non-claims. r[materialize_wrpc_telemetry.producer]

## Parent execution and publication

- [x] [serial] Materialize the standalone emitter and inspect actual imports and type-only dependencies. r[materialize_wrpc_telemetry.producer]
  - Materialized through Octet's `wrpc-telemetry-reference` profile; the emitter imports the `wasi:otel/tracing` effect plus type-only `wasi:otel/types` and `wasi:clocks/wall-clock@0.2.3`, which the manifest now declares. See the [2026-11-18 integration run](evidence/integration-2026-11-18.md); the earlier `octet-report-not-admitted` stop is in the [producer record](../materialize-wrpc-producer/evidence/integration-2026-11-18.md).
- [x] [serial] Run safe span, secret-redaction and 33rd-span queue rejection on the consumer's actual UCAN host. r[materialize_wrpc_telemetry.emitter]
  - Molten `7e158a95c` ran all three modes on these exact bytes with signed UCAN admission; the consumer's record is cited in the [integration run](evidence/integration-2026-11-18.md#consumer-execution).
- [x] [serial] Reject missing, mismatched, substituted, locally built and comparison artifact admission. r[materialize_wrpc_telemetry.producer]
  - `tests/wrpc-telemetry.rs` passed against the published bundle; see the [integration run](evidence/integration-2026-11-18.md#real-byte-acceptance).
- [x] [serial] Publish the source revision and record actual immutable output identities. r[materialize_wrpc_telemetry.producer]
  - Source revision `c376f843e` is published on `main`, and its output identities are recorded in the [integration run](evidence/integration-2026-11-18.md#bundles). The Octet pin awaits review in OnixResearch/octet#32.
