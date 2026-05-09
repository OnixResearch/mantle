## Why

The active OpenSpec queue is empty, but the full-source bootstrap claim is still gated by bridge outputs, placeholder-normalization stages, TinyCC/Mes runtime fragility, and prerequisite-only evidence. Those blockers are currently discoverable only by reading scattered specs, archived evidence, and bootstrap comments. That makes it easy for future work to overclaim `seed-full` or full-source bootstrap readiness after a successful queue drain.

## What Changes

- Add a deterministic blocker inventory/drift gate for the bootstrap chain.
- Emit a compact machine-readable and human-readable report of remaining gates.
- Fail closed when full-source promotion is claimed while known bridge, placeholder, fallback, or prerequisite-gated markers remain.
- Keep the gate advisory for ordinary edits unless explicitly invoked by a bootstrap/readiness rail.

## Scope

In scope: source-derived inventory over `bootstrap/`, canonical bootstrap specs, and checked-in evidence/metadata; a script or Rust cargo-script checker; positive and negative fixtures/transcripts; documentation of the report contract.

Out of scope: repairing TinyCC/GCC/musl blockers, promoting `seed-full`, running the full self-hosting proof, or changing blob/store identity semantics.

## Verification

Validation requires strict OpenSpec parsing, checker positive output on the current gated tree, a negative fixture or mutation proving fail-closed behavior when promotion is claimed with blockers present, and whitespace/version-control checks.
