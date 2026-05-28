## Design

The change stays in the pure planning core. The execution shell remains unchanged.

The native host planner already has the raw Cargo unit graph available. It derives a deterministic selected-host key set from raw Cargo units whose target kind is `custom-build` or `proc-macro`, keyed by `(package_id, target_name, target_kind)`. Custom-build Cargo target names such as `build-script-main` are normalized to Mantle's `build-script-build` package-target identity before matching. During native host package traversal, Mantle only materializes host target summaries whose keys are in that selected set.

This preserves the Cargo-selected boundary:

- selected proc-macro and custom-build units remain in the native host graph;
- same-package custom-build metadata can still flow into selected same-package proc-macro units;
- target units still consume proc-macro host artifacts through selected dependency/consumer facts;
- unselected manifest-visible proc-macro packages no longer become producer units or false execution blockers.

The native host planner still fail-closes when selected host-unit facts are missing or inconsistent. This is not a full Cargo scheduler claim.
