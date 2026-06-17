## ADDED Requirements

### Requirement: First-stage musl libgcc_eh unwind binding

r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_libgcc_eh_unwind] Mantle MUST bind first-stage source-root musl target `-lunwind` requests to a private target GCC unwind archive, preferring `libgcc_eh.a` when it is present.

#### Scenario: Unwind archive is selected from libgcc_eh

GIVEN the generated first-stage musl target wrapper selects a source-root target GCC CRT directory
WHEN it prepares private runtime libraries
THEN it MUST choose `libgcc_eh.a` as the unwind archive when that file exists.
AND it MUST copy the selected unwind archive to the private runtime directory as `libunwind.a`.

#### Scenario: Libgcc aliases remain available

GIVEN the generated first-stage musl target wrapper prepares private runtime libraries
WHEN Rust target links request libgcc-style aliases
THEN Mantle MUST continue to expose `libgcc.a` through the private runtime directory.
AND the private unwind binding MUST NOT expose generic target libraries globally.

#### Scenario: Libgcc fallback remains available for folded toolchains

GIVEN the selected target GCC CRT directory lacks `libgcc_eh.a`
AND `libgcc.a` is present
WHEN the generated first-stage musl target wrapper prepares private runtime libraries
THEN it MAY use `libgcc.a` as the private `libunwind.a` source.
AND later link failure from missing unwind symbols MUST remain a fail-closed build result.
