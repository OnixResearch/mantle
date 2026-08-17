# Operator Diagnostics Specification

## Purpose

Define provider-neutral remote-execution telemetry and exporter isolation.

## Requirements

### Requirement: Remote execution emits a canonical bounded telemetry model [r[operator_diagnostics.remote_execution_telemetry]]

Mantle MUST define provider-neutral bounded telemetry events and metric descriptors for route decisions, queue admission, assignment, attempt transitions, retry and stale-fence rejection, execution result, transfer demand/progress/resume/cutoff/fallback, output admission, and publication when those events occur. Event semantics MUST be owned by Mantle rather than by a Prometheus, OTLP, database, cloud, or dashboard adapter.

#### Scenario: Remote lifecycle events share stable semantics

- GIVEN a remote job moves through planning, assignment, execution, transfer, admission, and optional publication
- WHEN Mantle emits telemetry
- THEN events MUST use stable schema, phase, result, route, retry, transfer, and reason classes with bounded attributes
- AND equivalent lifecycle facts MUST normalize to equivalent event meaning regardless of exporter choice.

#### Scenario: Metrics keep bounded cardinality

- GIVEN telemetry includes job, attempt, worker, output, path, trace, error, ticket, key, or provider facts
- WHEN Mantle maps events to metric labels
- THEN labels MUST be limited to declared low-cardinality classes
- AND raw ids, store paths, output names, trace ids, bearer material, key material, credentials, and arbitrary error text MUST NOT become metric label values.

#### Scenario: Telemetry remains a diagnostic non-claim

- GIVEN telemetry reports a successful phase, transfer, or admitted result
- WHEN an operator or evidence summary cites it
- THEN the claim MUST be limited to the recorded runtime event and bound identities
- AND telemetry alone MUST NOT prove compiler correctness, source reproducibility, release eligibility, CI success, or physical-target determinism.

### Requirement: Telemetry exporters are optional isolated shells [r[operator_diagnostics.telemetry_exporter_isolation]]

Mantle MAY provide Prometheus and OTLP adapters over the canonical telemetry model, but exporter configuration MUST be typed, bounded, provider-neutral at the core boundary, and disabled unless explicitly selected. Exporter delay, backpressure, rejection, or outage MUST NOT change scheduler safety, build result, transfer identity, or output-admission truth.

#### Scenario: Configured exporter receives bounded telemetry

- GIVEN an operator explicitly enables a supported exporter with valid typed policy
- WHEN Mantle emits canonical remote events
- THEN the shell adapter MAY expose or export bounded normalized telemetry
- AND exporter-specific endpoints, credentials, batching, and network I/O MUST remain outside pure decision kernels.

#### Scenario: Exporter outage preserves build truth

- GIVEN a Prometheus endpoint cannot bind or an OTLP collector is unavailable, slow, or rejects a batch
- WHEN a remote build otherwise succeeds or fails
- THEN Mantle MUST preserve the original build and admission result and report exporter degradation separately
- AND it MUST drop, retry, or backpressure telemetry only within named bounded policy.

#### Scenario: Required observability evidence fails narrowly

- GIVEN an explicit operator policy requires durable observability evidence for a separate claim
- WHEN immutable log persistence or exporter delivery lacks the required receipt
- THEN Mantle MUST block only that observability-evidence claim with a deterministic diagnostic
- AND it MUST NOT rewrite an already established build or output-admission fact.
