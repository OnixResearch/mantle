## MODIFIED Requirements

### Requirement: Source-built native closure materialization

r[rust_package_planning.source_built_toolchain_closure.native_materialization] Mantle MUST materialize an explicit source-built native toolchain closure manifest only from digest-bound required members whose provider-root metadata authorizes the root for the requested host or target role, including the source-root musl layout only when its target matches that role's expected triple, and fail closed when the current provider/root lacks host or target native closure inputs.

#### Scenario: Complete source-root musl host fixture emits zero-seed manifest

GIVEN a musl-host Rust provider identity
AND source-root musl host and target roots expose target-prefixed helpers plus runtime/startup files
WHEN Mantle collects and materializes native closure candidates from those roots
THEN the manifest MUST contain no seed exceptions.
AND host `cc`, host `ld`, and host runtime members MUST point at source-root musl layout paths.
AND every manifest member MUST include a BLAKE3 content digest and source/build-receipt identity.

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
