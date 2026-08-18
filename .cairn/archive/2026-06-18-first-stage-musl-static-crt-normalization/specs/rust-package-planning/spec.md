## ADDED Requirements

### Requirement: First-stage musl static CRT normalization

r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_static_crt_normalization] Mantle MUST pair first-stage source-root musl static link mode normalization with a matching non-PIE musl startup object.

#### Scenario: Static-pie downgrade uses non-PIE startup

GIVEN the generated first-stage musl target wrapper receives `-static-pie` and `rcrt1.o`
WHEN it delegates to the source-root musl target GCC as a non-PIE static link
THEN it MUST pass `-static` and private `crt1.o` instead of `-static-pie` and `rcrt1.o`.
AND it MUST continue to pass private `crti.o`, `crtn.o`, `crtbeginS.o`, and `crtendS.o` paths.

#### Scenario: Startup normalization is private to downgraded links

GIVEN the generated first-stage musl target wrapper does not observe `-static-pie`
WHEN it maps musl startup objects
THEN it MUST preserve private `rcrt1.o` mapping for `rcrt1.o` inputs.
AND it MUST NOT expose a global `cc` alias or change generic host aliases.
