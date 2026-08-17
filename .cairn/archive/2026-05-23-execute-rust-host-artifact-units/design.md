# Design: bounded Rust host-artifact unit execution

## Execution model

The rail consumes the existing `unit_derivation_graph` receipt. It partitions ready graph nodes into:

1. supported host units: `execution_kind=host` with `target_kind=proc-macro` or `custom-build`;
2. supported target units: `execution_kind=target` with `target_kind=lib` or `bin`.

Host units execute first with the existing per-unit `rustc` execution receipt path. The rail then maps each successful host unit to its produced artifact:

- proc-macro host units produce the platform dynamic library emitted by rustc;
- custom-build units produce the build-script executable emitted by rustc and retain their generated metadata summary from planning evidence.

Target units are then executed in dependency order. Before invoking a target `rustc`, the rail rewrites every consumed host artifact placeholder to the matching host-produced path and rewrites matching dependency/`--extern` placeholders when the host artifact is also represented as a dependency artifact.

## Receipt shape

Add a host-artifact topology receipt containing:

- schema version;
- execution status;
- bounded claim text;
- ordered host and target `RustUnitExecutionReceipt` entries;
- optional blocker;
- stable self-reference-safe receipt hash.

The CLI receipt preserves both the captured `rust_plan` receipt and the host-artifact execution receipt.

## Fail-closed boundaries

The rail blocks before affected target execution when:

- `unit_derivation_graph.ready=false`;
- a target consumes a host artifact with no supported host producer in the graph;
- a host unit fails or emits no matching artifact;
- host artifacts are missing/unreadable after binding;
- dependency producers or target topology order cannot be resolved;
- unsupported target kinds/modes or cycles are encountered.

No ambient Cargo target directory or cache lookup is allowed.
