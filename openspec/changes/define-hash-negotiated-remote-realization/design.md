## Context

Mantle should avoid prematurely becoming a multi-machine orchestration product. The useful primitive is a deterministic content negotiation protocol: send identities, transfer missing content, run one derivation realization, and return output/proof identities.

## Goals / Non-Goals

**Goals:**
- Define the provider-neutral remote realization handshake.
- Keep derivation realization as the schedulable unit.
- Require digest verification before execution and before accepting outputs.

**Non-Goals:**
- Select a concrete network transport.
- Add arbitrary sub-action scheduling.
- Add SaaS control-plane semantics.

## Decisions

### 1. Identity-first handshake

**Choice:** A scheduler sends realization key, recipe identity, declared input identities, platform/profile facts, and declared capabilities before transferring content.

**Rationale:** Workers can answer with the exact missing hashes, minimizing transfer and making cache state explicit.

### 2. Worker-owned missing set

**Choice:** The worker determines which blobs, directories, recipes, and proof inputs it lacks and returns a missing set keyed by digest and kind.

**Rationale:** This supports heterogeneous caches and avoids assuming shared storage.

### 3. Receipts bind negotiated inputs

**Choice:** Returned realization receipts include the negotiated input digest set and output digest set.

**Rationale:** Remote execution evidence must be replayable and auditable.

## Risks / Trade-offs

**Protocol ossification** → version handshake structs and keep transport adapters separate.

**Digest mismatch during transfer** → fail before execution and classify the provider/worker response as invalid evidence.
