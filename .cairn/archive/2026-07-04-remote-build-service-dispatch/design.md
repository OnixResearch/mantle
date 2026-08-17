## Context

Mantle's lazy scheduler already deduplicates goals by derivation key and dispatches ready work through a `BuildService` boundary. Remote execution currently bypasses that boundary, so it cannot participate in normal dependency scheduling or per-goal routing.

## Decisions

### 1. Remote execution is a build-service adapter

**Choice:** Introduce a remote build service that accepts `BuildRequest`-equivalent concrete action data, performs remote protocol dispatch in the shell, and returns imported output nodes through the same finish path as local builds.

**Rationale:** The scheduler should not care whether a ready goal is realized locally or remotely once route admission has selected a concrete executor.

### 2. Request normalization stays in the functional core

**Choice:** Build request normalization, remote build key computation, upload manifest planning, and response-admission decisions are pure deterministic functions. Transport, process spawning, store I/O, and ticket persistence stay in shell code.

**Rationale:** Deduplication and negative tests need to exercise logic without a live remote machine.

### 3. Remote failures are phase-classified

**Choice:** Remote dispatch returns classified failures for transport setup, authorization, request validation, input sync, build execution, transfer, and output import. Fallback to local execution is allowed only when route policy names that retry class as fallback-eligible.

**Rationale:** A resource-access failure and an output-trust failure require different operator action.

## Risks / Trade-offs

- `BuildService` output shape must carry enough metadata for artifact attestations and transfer reports without bloating the scheduler.
- Retrying locally after some remote phases can mask policy errors unless fallback is explicit.
