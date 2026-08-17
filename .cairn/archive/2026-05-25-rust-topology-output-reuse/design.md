## Context

Mantle's Rust planner now owns native package/target facts, native unit graph facts, native host-unit graph facts, explicit unit derivations, host-artifact execution, build-script metadata binding, and unified topology execution. The accepted spec already requires topology output reuse to explain rebuild versus reuse with explicit receipt material.

This change introduces the bounded reuse seam for the existing topology execution output root. It does not introduce a general remote cache, scheduler, substitute protocol, Cargo target-dir compatibility, or broader Rust/Cargo support.

## Decisions

### 1. Treat previous topology receipts as reviewable cache identity material

**Choice:** Persist and reread prior per-unit execution receipt material under the execution output root and compare it against the current explicit unit execution identity before deciding to reuse.

**Rationale:** The execution output root is already an explicit CLI input. Using receipt material from that root keeps reuse local, deterministic, and reviewable, while avoiding ambient Cargo target directories or implicit caches.

### 2. Reuse only on full explicit-input match

**Choice:** A prior output is reusable only when the unit identity, source closure digest, dependency artifact digests, host artifact digests, toolchain identity, `rustc` argument digest, declared output paths, and current output artifact BLAKE3 digests match the current unit's expected material.

**Rationale:** Partial matches create ambiguous cache claims. Requiring all explicit identity inputs to match makes the reuse reason auditable and preserves the existing Cargo-free bounded claim.

### 3. Fail closed on stale prior evidence

**Choice:** If prior receipt evidence exists but is unreadable, malformed, missing a declared output, digest-mismatched, or describes different current inputs, topology execution returns a structured stale-cache blocker before invoking `rustc` for that unit.

**Rationale:** Silent fallback from stale cache evidence to rebuild would hide the precise cache-identity failure. A deterministic blocker lets operators inspect and intentionally clear or repair the output root.

### 4. Keep the first slice local and topology-only

**Choice:** Limit this change to the existing `rust-plan --execute-topology` rail and local execution output roots.

**Rationale:** Remote caches, substitution, eviction policy, concurrent cache mutation, and general scheduling are valuable later seams, but they should not be coupled to the first receipt-bound reuse proof.

## Risks / Trade-offs

- Strict stale-cache blockers can require users to clear an execution output root before rerunning after source or toolchain changes. That is intentional until a reviewed cache invalidation command exists.
- The first implementation may duplicate some identity comparison logic from per-unit execution receipts; keep the comparison helper pure and testable so later cache-store work can reuse it.
- Receipt schema changes can affect JSON consumers; keep additions additive where possible and preserve bounded non-claims.
