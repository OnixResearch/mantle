## Context

Mantle's Rust planning receipts now include Mantle-owned native package/target facts, native unit graph facts, native host-unit graph facts, unit derivation graph evidence, and multiple bounded execution rails. The host-artifact-specific rail already receives `native_host_unit_graph_planning` and rejects missing/non-native host material before execution. The unified `--execute-topology` rail still takes only `unit_derivation_graph`, executes supported host units first, runs build-script metadata, binds host artifacts, binds target dependencies, and executes targets in dependency order.

This change makes unified topology execution reuse the same native-host validation boundary as host-artifact topology execution, without broadening the supported Rust/Cargo surface.

## Decisions

### 1. Gate unified topology with native host graph evidence

**Choice:** Change the unified topology execution API so it receives `&NativeHostUnitGraphPlanningSummary` alongside `&UnitDerivationGraphSummary`.

**Rationale:** The CLI's retained `RustPlanReceipt` already contains both receipt sections. Passing both into the executor keeps the public JSON shape stable while making the imperative execution shell prove host facts came from Mantle-owned native planning before invoking host or target `rustc`.

### 2. Reuse deterministic host-artifact validation semantics

**Choice:** Apply the same fail-closed classes used by native host-artifact topology execution where possible: blocked native host graph, blocked derivation graph, missing native host derivation, non-native host derivation, missing native host consumer derivation, missing native host artifact binding, and non-native host artifact consumer.

**Rationale:** Reusing the validation contract avoids parallel blocker vocabularies and makes `--execute-host-artifact-topology` and `--execute-topology` disagree only in their scheduling/binding scope, not in their native-host trust boundary.

### 3. Keep this slice bounded to the existing supported topology rail

**Choice:** Do not add a new scheduler, feature resolver, test/doctest/example support, native-link probing, or broader Cargo compatibility.

**Rationale:** The value of this slice is closing the main rail's native-host evidence seam. Broader Rust execution behavior should remain separate Cairn changes with their own requirements and fixtures.

## Risks / Trade-offs

- The unified executor may reject mixed graphs that previously executed if their host material is not represented in ready native host-unit facts. This is intentional fail-closed behavior.
- Target-only topologies should remain accepted when no host units or host consumers exist, while mixed host+target graphs require native host readiness.
- Existing receipt consumers may see new blocker classes for `--execute-topology`; this is the desired deterministic evidence rather than hidden fallback behavior.
