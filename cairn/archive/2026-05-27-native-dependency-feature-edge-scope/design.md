# Design

Native package planning currently computes selected features from the top-level `rust-plan` invocation for every package. That is too broad for transitive dependencies: a dependency edge can set `default-features = false`, and Cargo's unit graph already records the selected feature set for each package under the current build invocation.

The change adds a deterministic selected-feature index keyed by Cargo package id:

1. Read `units[].pkg_id` and `units[].features[]` from the captured unit graph.
2. Union and sort features per package id.
3. When planning a native package, prefer the unit-graph feature set for that package id; only fall back to invocation-derived root features when the package is absent from the unit graph.
4. Reuse the selected package feature set for optional dependency decisions.
5. For build dependencies, stop treating `features`, `default-features`, and `optional` as immediate unsupported options. Resolve the source edge through the same bounded path/vendored-registry fragment, and let the selected-feature index describe the dependency package's actual features.

This remains oracle-bounded: the unit graph is still retained as review evidence, and the native receipt keeps `not-full-cargo-feature-resolution` as a non-claim.
