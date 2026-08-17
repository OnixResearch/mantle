# Design: Native Rust feature resolution

## Context

Feature resolution controls unit identity, rustc `--cfg feature=...`, dependency inclusion, and crate metadata disambiguation. Missing or wrong features cause false builds or confusing rustc errors.

## Decisions

### 1. Implement a pure fixed-point resolver

**Choice:** Resolve features over an explicit package/dependency graph with deterministic queues and bounded iteration checks.

**Rationale:** Feature closure is graph logic and should be tested without filesystem or rustc.

### 2. Start with resolver v2 build/normal separation

**Choice:** Model normal, build, and proc-macro host roles explicitly for the bounded workspace/profile surface Mantle supports.

**Rationale:** Host/target feature leakage changes unit identity and artifact compatibility.

### 3. Treat unsupported target cfg dependencies as blockers first

**Choice:** Until full cfg evaluation exists, unsupported `target.'cfg(...)'.dependencies` forms fail closed.

**Rationale:** Platform-specific dependencies cannot be ignored safely.

## Risks / Trade-offs

- Cargo feature behavior has many edge cases; oracle fixtures must stay broad.
- Initial support may block crates that depend on complex target cfg tables.
