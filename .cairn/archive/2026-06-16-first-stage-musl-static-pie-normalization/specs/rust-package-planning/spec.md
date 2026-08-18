## ADDED Requirements

### Requirement: First-stage musl static-pie normalization

r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_static_pie_normalization] Mantle MUST normalize first-stage source-root musl target wrapper links away from unsupported static PIE mode when the selected seed libc only supports non-PIE static links.

#### Scenario: Rust static-pie request uses source-root static link

GIVEN the generated first-stage musl target wrapper receives `-static-pie`
WHEN it delegates to the source-root musl target GCC
THEN it MUST pass `-static` instead of `-static-pie`.
AND the normalization MUST preserve existing CRT path mapping and runtime object injection.

#### Scenario: Normalization remains private to the first-stage wrapper

GIVEN first-stage source-root musl target link normalization is enabled
WHEN the generated script exits
THEN Mantle MUST NOT expose a global `cc` alias or alter generic host aliases.
AND the normalization MUST apply only through the generated private target wrapper directory.
