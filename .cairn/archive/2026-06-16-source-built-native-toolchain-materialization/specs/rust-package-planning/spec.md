## ADDED Requirements

### Requirement: Source-built native closure materialization

r[rust_package_planning.source_built_toolchain_closure.native_materialization] Mantle MUST materialize an explicit source-built native toolchain closure manifest only from digest-bound required members and fail closed when the current provider/root lacks host or target native closure inputs.

#### Scenario: Complete provider root emits zero-seed manifest

GIVEN a concrete provider root contains a source-built Rust compiler, host C/linker/runtime/sysroot members, and target-prefixed musl helper members
WHEN Mantle materializes a source-built native closure manifest from that root
THEN the manifest MUST contain only source-built members.
AND it MUST contain no seed exceptions.
AND every member MUST include a BLAKE3 content digest and source/build-receipt identity.

#### Scenario: Missing host linker runtime fails closed

GIVEN a provider root contains a source-built Rust compiler but lacks host C/linker/libc/startup/runtime members
WHEN Mantle attempts to materialize a source-built native closure manifest
THEN it MUST fail before writing a claiming manifest.
AND the diagnostic MUST name the missing host native closure surface.

#### Scenario: Missing target helper fails closed

GIVEN a provider root lacks one or more target-prefixed musl helpers
WHEN Mantle attempts to materialize a source-built native closure manifest
THEN it MUST fail before writing a claiming manifest.
AND the diagnostic MUST name the missing target helper roles.

#### Scenario: Fixed-point proof remains authoritative

GIVEN Mantle has materialized a zero-seed native closure manifest
WHEN a Cargo-free fixed-point proof is run with `--toolchain-closure <manifest>`
THEN only the enforced fixed-point summary MAY retire `not-source-built-toolchain-closure`.
AND manifest materialization alone MUST NOT be reported as release reproducibility or full Cargo compatibility evidence.
