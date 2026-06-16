## MODIFIED Requirements

### Requirement: Source-built native closure materialization

r[rust_package_planning.source_built_toolchain_closure.native_materialization] Mantle MUST materialize an explicit source-built native toolchain closure manifest only from digest-bound required members whose provider-root metadata authorizes the root for the requested host or target role, including the source-root musl layout only when its target matches that role's expected triple, and fail closed when the current provider/root lacks host or target native closure inputs.

#### Scenario: Complete provider root emits zero-seed manifest

GIVEN a concrete provider root contains a source-built Rust compiler, host C/linker/runtime/sysroot members, and target-prefixed musl helper members
AND the host and target roots advertise matching source-built native capabilities in their provider metadata
WHEN Mantle materializes a source-built native closure manifest from that root
THEN the manifest MUST contain only source-built members.
AND it MUST contain no seed exceptions.
AND every member MUST include a BLAKE3 content digest and source/build-receipt identity.

#### Scenario: Source-root musl can satisfy a musl host root

GIVEN a Rust provider host triple is `x86_64-unknown-linux-musl`
AND a root advertises source-root metadata for `x86_64-linux-musl`
WHEN Mantle collects host native closure members from that root
THEN it MUST use the source-root musl layout for `cc`, `ld`, `crt1.o`, `libgcc_s.so.1`, and `libc.so`.

#### Scenario: Target-only source-root metadata cannot satisfy GNU host root

GIVEN a root advertises source-root metadata for `x86_64-linux-musl`
WHEN Mantle is asked to use that root as the host-native root for a GNU-host Rust provider
THEN it MUST fail before collecting host C/linker/libc/startup/runtime member paths.
AND the diagnostic MUST name the host-root capability mismatch.

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
