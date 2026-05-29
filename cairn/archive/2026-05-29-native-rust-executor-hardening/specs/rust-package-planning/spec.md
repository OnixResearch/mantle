## ADDED Requirements

### Requirement: Native Rust executor hardening

r[rust_package_planning.native_executor_hardening] Mantle MUST execute and cache native Rust units only from explicit receipt-bound inputs, artifacts, environment, and toolchain identity.

#### Scenario: cache hit requires full receipt match

GIVEN a prior unit execution receipt and output artifact exist
WHEN Mantle considers reusing the output
THEN it MUST verify unit identity, rustc argument digest, declared input digests, dependency and host artifact digests, environment digest, toolchain identity, and output digest before reporting a cache hit.

#### Scenario: unresolved artifacts block before rustc

GIVEN a unit has dependency or host artifact placeholders
WHEN Mantle prepares rustc execution
THEN every placeholder MUST resolve to a declared existing artifact with matching digest before rustc is invoked.

#### Scenario: failure diagnostics are deterministic

GIVEN rustc execution or preflight validation fails
WHEN Mantle writes the execution receipt
THEN it MUST include a deterministic failure class and redacted diagnostics sufficient for review.
