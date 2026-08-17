# Design: Native Rust executor hardening

## Context

The executor is the boundary where planned units become artifacts. It must not repair missing data from ambient Cargo caches or stale target directories.

## Decisions

### 1. Make receipt validation the cache gate

**Choice:** Reuse prior outputs only when unit identity, rustc args digest, declared input digests, host artifact digests, env digest, toolchain identity, and output digests match.

**Rationale:** Cache reuse without all material is unsound.

### 2. Validate artifact paths before rustc

**Choice:** Dependency and host artifact placeholders must resolve to existing declared files with matching digests before invoking consumer rustc.

**Rationale:** rustc error messages are too late and often misleading.

### 3. Keep diagnostics deterministic and redacted

**Choice:** Failure receipts include stable blocker classes and redacted stderr snippets, not ambient env dumps or temp-only paths as hash material.

**Rationale:** Receipts must be reviewable and replayable.

## Risks / Trade-offs

- Stricter validation can expose new blockers before rustc.
- Digesting more material may make receipt schema migrations necessary.
