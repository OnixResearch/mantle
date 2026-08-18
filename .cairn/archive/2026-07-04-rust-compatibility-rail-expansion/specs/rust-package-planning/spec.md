## ADDED Requirements

### Requirement: Native Rust planning reports compatibility surface matrix results

r[rust_package_planning.compatibility_surface_matrix] Mantle MUST classify native Rust planning and execution results against a bounded compatibility surface matrix when `rust-plan` is used for Cargo-free evidence. Supported surfaces MAY report `cargo-free-bounded-topology`; unsupported or incomplete surfaces MUST report deterministic `blocked-unsupported-surface` diagnostics and MUST NOT invoke Cargo, ambient caches, network sources, or hidden build orchestration to satisfy the matrix.

#### Scenario: supported matrix surface is cargo-free

GIVEN a matrix fixture uses a Rust surface that Mantle's native planner supports
AND all source, toolchain, feature, host artifact, native-link, and dependency material is declared
WHEN `mantle rust-plan --no-cargo-oracle` plans or executes that fixture
THEN Mantle MAY report `cargo-free-bounded-topology`
AND the receipt MUST bind the surface id, source closure digest, unit graph facts, and non-claims.

#### Scenario: unsupported matrix surface blocks without Cargo fallback

GIVEN a matrix fixture uses a Rust surface outside Mantle's native planning or execution subset
WHEN `mantle rust-plan --no-cargo-oracle` evaluates that fixture
THEN Mantle MUST report `blocked-unsupported-surface` with a stable blocker class
AND it MUST NOT run Cargo or read undeclared Cargo registry, git, target, or package-manager caches to complete the plan.

#### Scenario: native-link surfaces are explicit

GIVEN a Rust package uses `links`, `pkg-config`, `cargo:rustc-link-lib`, `cargo:rustc-link-search`, or native C compilation
WHEN Mantle classifies the native Rust planning surface
THEN the receipt MUST either model the link metadata and declared native inputs explicitly or fail with a native-link blocker
AND it MUST NOT infer host library availability from ambient system paths.
