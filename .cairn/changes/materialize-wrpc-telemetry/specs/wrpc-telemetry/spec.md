# Optional telemetry producer

## ADDED Requirements

### Requirement: Actual pinned tracing emitter [r[materialize_wrpc_telemetry.emitter]]

The optional emitter MUST import the exact selected wasi:otel/tracing@0.2.0-rc.2
contract and export run(mode:u32)->u32. Every emitted span MUST pair the actual
host current-span-context, on-start and on-end calls. Mode0 MUST emit one safe
span; mode1 MUST include the reviewed secret attribute negative input; mode2 MUST
emit33 separately paired spans. Clock effects MUST NOT be introduced for timestamps.

#### Scenario: Queue-bound negative input is real

- GIVEN the selected standalone emitter and independently admitted actual host
- WHEN mode2 runs
- THEN the emitter attempts33 real start/end pairs
- AND the consumer MUST enforce its32-span budget rather than accepting a mock result.

### Requirement: Separate immutable producer identity [r[materialize_wrpc_telemetry.producer]]

Mantle MUST retain actual exporter receipts, exact WIT/source/cohort identities,
all member hashes/sizes, runtime profile and confined capability layout under the
separate wrpc-telemetry-reference-bundle output. Comparison-only composition MUST
NOT replace the open emitter or claim trusted host/UCAN/telemetry export evidence.
No hello import policy or existing deny-all WASI defaults may be expanded.

#### Scenario: Comparison cannot satisfy host evidence

- GIVEN the real source-built sink and exact emitter bound for producer smoke
- WHEN the comparison returns42
- THEN its artifacts remain explicitly comparison-only
- AND the consumer still requires the separately authorized actual host and file capability.
