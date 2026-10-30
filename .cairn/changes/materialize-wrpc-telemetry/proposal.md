# Materialize optional OpenTelemetry-WASI emitter

## Why

The approved hello cohort has no OpenTelemetry-WASI interface. Optional telemetry
requires its own selected immutable upstream WIT, reviewed import boundary and
real emitter artifact, not a generic queue or native OTLP mislabeled WASI.

## What changes

Materialize a separately governed optional emitter from the approved wasi-otel
commit, exact tracing WIT and isolated Wasmtime33 cohort. Retain positive,
secret-attribute and queue-budget modes for actual consumer execution. Use a real
source-built tracing sink only for explicitly comparison-only producer smoke.

## Scope

No expansion of the hello profile or WASI defaults. The consumer owns actual
UCAN admission, trusted context, redaction, lifecycle/queue enforcement and
capability-local diagnostic export. Publication and all execution remain parent-owned.
