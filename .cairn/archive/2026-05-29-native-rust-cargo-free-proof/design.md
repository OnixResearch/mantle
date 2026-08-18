# Design: Bounded Cargo-free Rust topology proof

## Context

Mantle's current self-probe shows direct rustc topology can build hundreds of units, but it still starts from Cargo oracle material. This proof demonstrates the native planner path on a generated multi-crate path workspace and explicitly does not claim Mantle/Crunch self-build evidence.

## Decisions

### 1. Separate fast preflight from full proof

**Choice:** Provide a fast check for tools/fixtures and an explicit full bounded proof for the generated multi-crate path workspace.

**Rationale:** Developers need quick feedback; release evidence needs the full run.

### 2. Disable Cargo in the proof environment

**Choice:** Use a PATH shim or policy guard that makes any Cargo invocation fail and records that guard in evidence.

**Rationale:** The proof must not accidentally fall back to Cargo.

### 3. Write durable audit bundles

**Choice:** Store receipts, stdout/stderr, tool identities, source digest, output digests, blocker summaries, and final binary checks under a stable proof directory.

**Rationale:** Reviewers need direct evidence, not memory of a prior run.

## Risks / Trade-offs

- Full bounded proof remains host-sensitive.
- This proof intentionally targets a bounded workspace subset before full Cargo ecosystem parity or Mantle/Crunch self-build evidence.
