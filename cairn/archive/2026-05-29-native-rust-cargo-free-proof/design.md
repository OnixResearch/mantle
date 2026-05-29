# Design: Cargo-free self-build proof

## Context

Mantle's current self-probe shows direct rustc topology can build hundreds of units, but it still starts from Cargo oracle material. The proof must demonstrate the native planner path.

## Decisions

### 1. Separate fast preflight from full proof

**Choice:** Provide a fast check for tools/fixtures and an explicit long-running proof for the full workspace.

**Rationale:** Developers need quick feedback; release evidence needs the full run.

### 2. Disable Cargo in the proof environment

**Choice:** Use a PATH shim or policy guard that makes any Cargo invocation fail and records that guard in evidence.

**Rationale:** The proof must not accidentally fall back to Cargo.

### 3. Write durable audit bundles

**Choice:** Store receipts, stdout/stderr, tool identities, source digest, output digests, blocker summaries, and final binary checks under a stable proof directory.

**Rationale:** Reviewers need direct evidence, not memory of a prior run.

## Risks / Trade-offs

- Full proof may be long and host-sensitive.
- Initial proof may target a bounded workspace subset before full Cargo ecosystem parity.
