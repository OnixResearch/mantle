## Context

The archived `replace-cargo-rust-planner` change established the trusted planning side of Mantle Rust package support:

```text
cargo metadata + cargo build --unit-graph
  -> normalized Rust plan receipt
  -> explicit source closure
  -> explicit unit_derivation_graph
  -> host/proc-macro receipt nodes
  -> deterministic unsupported-boundary blockers
```

The accepted spec already requires per-unit output receipts to bind unit identity, source closure digest, dependency artifact digests, toolchain identity, `rustc` argument digest, and output artifact digest. The implementation currently emits reviewable unit derivation plans, but it does not yet execute those plans through a Mantle-owned build path.

## Decisions

### 1. Execute only ready supported unit derivation nodes first

**Choice:** The first execution rail consumes `unit_derivation_graph` entries only when the graph is ready and the unit is in the currently supported `lib`/`bin` subset.

**Rationale:** This preserves the fail-closed boundary established by the planner and avoids turning unsupported Cargo behavior into an implicit Cargo fallback.

### 2. Treat receipt material as the execution contract

**Choice:** Execution uses the planned `rustc` args, environment, source-closure digest, dependency artifact placeholders, host artifact references, and declared outputs as the authoritative input contract.

**Rationale:** The goal is to prove that Mantle can build from its own reviewable receipts. Recomputing hidden state during execution would reintroduce the same cache/orchestrator ambiguity the planner removed.

### 3. Record output and rebuild/reuse evidence per unit

**Choice:** Each executed unit emits a deterministic execution receipt with output artifact digests, selected toolchain identity, input digests, `rustc` argument digest, execution status, and rebuild/reuse reason.

**Rationale:** The accepted planning spec requires receipts that explain cache identity and rebuild behavior. Execution without this evidence would produce artifacts but not a trustworthy build trail.

### 4. Keep host and dependency artifacts explicit

**Choice:** Target units may consume dependency and host artifacts only when those artifacts are present as explicit receipt-bound inputs. Missing host/proc-macro/build-script artifacts produce deterministic blockers.

**Rationale:** Host/target confusion is a correctness risk. The execution rail must not synthesize or discover those artifacts from ambient Cargo target directories.

### 5. Bound the first claim

**Choice:** The initial claim is limited to executing explicit supported Rust unit nodes. It does not claim full Cargo replacement, doctest/test/example support, native-link probing correctness, or rustc/compiler correctness.

**Rationale:** Maintaining bounded claims makes the resulting evidence reviewable and prevents a small execution slice from overrepresenting ecosystem compatibility.

## Risks / Trade-offs

- Direct `rustc` execution may expose missing flags or environment assumptions that Cargo supplied implicitly; those should become blockers or explicit receipt fields, not hidden Cargo calls.
- Dependency artifact scheduling may require a minimal topological execution order before multi-unit workspaces can build end-to-end.
- Host unit execution for build scripts and proc macros may need a separate first-class artifact materialization step before target units can consume them.
- Output path determinism and cleanup must be handled carefully so repeated executions produce stable receipts without trusting stale target directories.
