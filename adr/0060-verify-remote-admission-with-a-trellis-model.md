# ADR 0060: Verify remote admission with a Trellis model

## Status

Accepted

## Context

Mantle's remote-attempt core already makes pure decisions for assignment identity, fences, report idempotence, phase transitions, retry, and output-admission authorization. Tests cover these functions, but no machine-checked proof connects the full modeled transition relation to its safety properties.

The [Atom Reforged architecture](https://nrd.sh/blog/atom-reforged.html) uses small formal protocol models. Trellis already owns reusable Verus-checked logic and provides relevant fencing, lease, durable-job, and rejection-preservation patterns.

Mantle also has an accepted Kamacite and Valence path for Trellis proof evidence. That path preserves proof identity without giving proof sidecars runtime authority.

## Decision Drivers

- Prove the highest-value remote state safety properties.
- Keep the model small and reviewable.
- Keep Mantle runtime semantics owned by Mantle.
- Reuse Trellis proof infrastructure and existing evidence profiles.
- Make model drift fail closed.

## Decision

A paired Trellis change will add a product-neutral `fenced_attempt_admission` model with spec functions, executable functions, postconditions, and named proofs.

The first proof set covers stale and mismatched fence rejection, terminal-state closure, duplicate-event idempotence, conflicting-event rejection, completion linkage, representable fence advance, and rejection preservation.

Mantle will add a pure projection from admitted remote-attempt facts into the Trellis model. A bounded complete parity rail will compare Mantle and Trellis executable decisions across supported phase, report, fence, event, authorization, and result-linkage classes. Unmapped variants fail closed.

The durable attempt state records `progress_events_applied`. Legacy states decode a missing field as zero. Progress reports increment this bounded value.

The first oracle has 6,720 cases. The projection supports 5,882 cases and rejects 838 semantic differences with stable reason classes.

Trellis proof artifacts flow through Kamacite and Valence. Mantle binds accepted evidence through its existing opaque sidecar profile. Ordinary remote admission continues to use Mantle's Rust core.

Development can use the sibling `../trellis` checkout. Durable evidence and CI name pinned source, verifier, proof, policy, assumption, and receipt identities. Mantle gains no runtime dependency on a workspace-relative path.

## Alternatives Considered

### Model the whole remote-build system

Rejected because transport, persistence, cryptography, scheduling, and worker execution create an unbounded and weakly connected proof surface.

### Put Verus proofs inside Mantle

Rejected because Trellis owns reusable verified logic and the proof toolchain boundary.

### Use only TLA+ or Alloy

Deferred. A separate concurrency model can be useful later, but Trellis gives direct checked spec and executable functions for the current pure Rust decision seam.

### Link Trellis directly into runtime admission

Rejected because proof evidence must not become output authority or an ambient runtime dependency.

## Consequences

- Trellis and Mantle need paired Cairn changes and pinned evidence.
- Mantle gains an explicit projection and parity rail.
- New transition variants cannot claim proof coverage until the model is updated.
- Accepted proof evidence supports only the named abstract safety properties.
- Persistence, transport, cryptography, worker correctness, liveness, and release eligibility remain unproven.
