# Design: Native registry transitive producer coverage

## Context

The latest pushed-head self probe (`target/mantle-self-rust-plan-probe-after-push-d09d699b/receipt.json`) reports all native planning fragments ready but topology execution blocked by a missing producer for `itertools@0.10.5`.

Inspection shows:

- `native_package_target_planning.packages` contains `registry+https://github.com/rust-lang/crates.io-index#itertools@0.10.5` with a vendored manifest and one target.
- `unit_derivation_graph.derivations` contains `itertools@0.12.1` and `itertools@0.14.0`, but not `itertools@0.10.5`.
- A consumer unit (`vendor/snix-build`) declares a dependency artifact for `itertools@0.10.5`.

So the graph has enough native package/source evidence to build the producer, but the executable unit graph omits it while still reporting ready.

## Decisions

### 1. Close producer coverage during planning, not execution

**Choice:** Native unit/derivation graph planning must prove that every consumed dependency artifact has a matching supported producer unit, or emit a deterministic planning blocker.

**Rationale:** Topology execution should execute a proven graph. A missing supported producer discovered only during execution is too late and makes `unit_derivation_graph.ready=true` overclaim coverage.

### 2. Derive supported transitive producers from native package facts

**Choice:** When a dependency artifact points at a package with ready native package facts, ready source facts, and a supported `lib` target, planning may add or retain a producer unit for that package even if the package is only transitively reachable.

**Rationale:** Registry, path, and captured git dependencies are already normalized into native package/source facts. Producer coverage should be driven by those explicit facts, not by workspace-root status or ad hoc topology discovery.

### 3. Keep unsupported producer cases fail-closed

**Choice:** If the dependency artifact points at a package without native package facts, without ready source facts, without a supported `lib` target, or requiring unsupported host/test/feature behavior, planning must emit a stable blocker and keep downstream readiness false.

**Rationale:** The change should unblock supported registry producers like `itertools@0.10.5` without broadening to full Cargo scheduling.

### 4. Preserve provider-neutral source material

**Choice:** Producer units consume source material through existing native source facts and BLAKE3 source digests. They must not fetch registry/git data or inspect ambient Cargo caches.

**Rationale:** This continues the source-closure/snix-store direction: source identity and source bytes are explicit receipt inputs, not hidden host state.

## Risks / Trade-offs

- Adding transitive producers can expose the next unsupported dependency or host-artifact blocker. That is acceptable if the blocker is deterministic and earlier than rustc execution.
- Cargo oracle comparison may need tightening so Mantle does not silently invent producers that Cargo would not build for the selected feature set.
- The first implementation should stay scoped to supported `lib` producers for build-mode target dependencies; examples, benches, doctests, and general test scheduling remain out of scope.
