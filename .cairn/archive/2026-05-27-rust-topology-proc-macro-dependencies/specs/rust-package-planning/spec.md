## ADDED Requirements

### Requirement: Native proc-macro host dependency binding

r[rust_package_planning.native_proc_macro_host_dependency_binding] Mantle MUST bind native proc-macro host units with Cargo-equivalent compiler and dependency inputs before rustc execution.

#### Scenario: Proc-macro host unit receives compiler proc_macro extern

GIVEN a native package declares `[lib] proc-macro = true`
WHEN Mantle derives the proc-macro host rustc invocation
THEN Mantle MUST include the compiler-provided `proc_macro` extern input.
AND Mantle MUST NOT require a produced package artifact for `proc_macro`.

#### Scenario: Proc-macro host unit receives selected normal dependencies

GIVEN a native proc-macro package has selected normal dependencies such as `proc-macro2`, `quote`, or `syn`
WHEN Mantle plans native host-unit dependency artifacts
THEN Mantle MUST include Cargo-selected normal dependencies as host-unit dependency artifacts.
AND Mantle MUST NOT include unselected manifest-only dependency edges.
AND Mantle MUST bind selected artifacts to produced target library paths before executing the proc-macro host unit.

#### Scenario: Proc-macro dependency features reach rustc cfgs

GIVEN a native proc-macro dependency crate has selected features such as `syn/parsing` or `syn/visit-mut`
WHEN Mantle derives that dependency crate's native rustc invocation
THEN Mantle MUST pass the selected features as deterministic `--cfg feature=...` rustc args.
AND Mantle MUST preserve the no-feature default when no features are selected.

#### Scenario: Proc-macro dependencies run before proc-macro host unit

GIVEN a combined native topology contains a proc-macro host unit and its normal library dependency producers
WHEN Mantle orders the topology
THEN Mantle MUST schedule dependency library producers before the proc-macro host unit.
AND Mantle MUST preserve deterministic blocker reporting if a dependency producer is missing.

#### Scenario: Proc-macro frontier moves

GIVEN topology execution currently blocks while compiling `async-stream-impl` with unresolved `proc_macro` and dependency imports
WHEN proc-macro host dependency binding is applied
THEN self-probe verification MUST show that this unresolved-import blocker no longer stops the topology at `async-stream-impl`.
AND any remaining blocker MUST be recorded with deterministic class and evidence.
