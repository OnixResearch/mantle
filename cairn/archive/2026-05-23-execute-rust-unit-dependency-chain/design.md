## Context

The archived `execute-rust-unit-derivations` change proved that Mantle can invoke `rustc` for one ready supported unit using explicit derivation receipt material and can produce deterministic execution receipts. Multi-unit Rust workspaces still stop at dependency artifact placeholders such as `artifact:<package-id>:<crate>`, so a consumer cannot yet use a Mantle-built dependency artifact.

## Decisions

### 1. Start with a bounded dependency chain, not a scheduler

**Choice:** Execute the first ready target unit with declared dependency artifacts only when every dependency artifact can be matched to a supported producer unit in the same graph.

**Rationale:** This proves the next orchestration seam without adding a general topological scheduler or broad Cargo compatibility claims.

### 2. Producer receipts are the source of dependency artifacts

**Choice:** A consuming unit may receive an `--extern name=<path>` rewrite only from an artifact produced by the matched producer execution output.

**Rationale:** The dependency edge must be grounded in Mantle-owned receipts, not in ambient Cargo target directories or registry/git caches.

### 3. Keep dependency blockers deterministic

**Choice:** Missing producer units, missing producer output artifacts, stale or unreadable rewritten dependency paths, host artifact requirements, or unsupported producer/consumer shapes produce deterministic blockers before invoking the consumer `rustc`.

**Rationale:** Fail-closed dependency materialization prevents silent fallback to Cargo orchestration and keeps claim boundaries reviewable.

### 4. Receipts remain per-unit with chain evidence around them

**Choice:** The chain result records ordered unit execution receipts and a chain-level status/blocker rather than changing the existing per-unit execution receipt schema.

**Rationale:** Existing receipts already bind unit/toolchain/args/output identity. A thin chain receipt can explain the dependency edge without destabilizing the per-unit schema.

## Risks / Trade-offs

- Direct `rustc` dependency linking may expose additional Cargo-provided flags; those should become explicit fields or blockers in later changes.
- The first chain rail intentionally rejects host/proc-macro/build-script artifacts to avoid mixing host execution into this target dependency edge.
- General scheduling, reuse, and fan-out/fan-in graphs remain later work.
