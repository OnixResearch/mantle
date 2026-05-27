## Context

The unified topology scheduler already handles target library edges and target consumers of host artifacts. It does not yet model host-to-host producer ordering for proc-macro dependencies used by other host units. In the current self probe, `rustversion@1.0.22` is present as `rustversion:proc-macro:build`, but host consumers still fail because their dependency artifacts are resolved only through target `lib` producers.

## Decisions

### 1. Derive host dependency ordering from selected derivation artifacts

**Choice:** Build a small pure scheduling core from the selected `UnitDerivationGraphSummary`: host dependency artifacts first try target `lib` producers, then supported proc-macro host producers. Target dependencies populate the existing target prebuild set; proc-macro dependencies add host-to-host edges.

**Rationale:** The selected derivation graph is already the bounded source of truth for dependency artifacts. Following it avoids ambient manifest expansion and keeps behavior aligned with Cargo-selected units/features/platforms.

### 2. Bind proc-macro host outputs as host dependency artifacts

**Choice:** When executing host units, bind dependency artifacts from the union of produced target-library artifacts and produced proc-macro host artifacts.

**Rationale:** A host unit using a proc macro needs an `--extern` path to that produced host artifact, not a target `lib` path. Keeping a separate proc-macro artifact map avoids confusing all host outputs with target libraries.

### 3. Fail closed for unsupported host dependency producers

**Choice:** If a host-unit dependency artifact has neither a target `lib` producer nor a supported proc-macro host producer, keep returning a deterministic `missing-host-dependency-producer` blocker naming the dependency package.

**Rationale:** Build scripts and other host artifacts are not generally Rust library inputs. Silent fallback would hide missing modeling.

## Risks / Trade-offs

- Host packages with multiple proc-macro-like artifacts remain bounded by package identity because dependency artifacts currently identify packages, not individual host targets.
- Full Cargo host scheduling is still out of scope; this change covers selected proc-macro host dependencies needed by current native topology progress.
