## ADDED Requirements

### Requirement: Source-built provider Mantle binary warning frontier

r[rust_package_planning.source_built_toolchain_closure.mantle_bin_warning_frontier] Mantle MUST keep provider-backed Cargo-free topology execution from failing the native Mantle binary on unused-item warnings caused by test-only helpers or intentionally dormant provider/front-end/native-planner surfaces leaking into non-test compilation.

#### Scenario: Test-only imports do not warn in native binary execution

GIVEN provider-backed Cargo-free topology execution compiles the native Mantle binary
WHEN a helper import is used only by `#[cfg(test)]` code
THEN Mantle MUST scope that helper import to the test code rather than exposing it to the non-test binary build.
AND Mantle MUST NOT suppress the warning globally to hide unrelated unused imports.

#### Scenario: Test-only constants do not warn in native binary execution

GIVEN provider-backed Cargo-free topology execution compiles the native Mantle binary
WHEN a helper constant is used only by `#[cfg(test)]` code
THEN Mantle MUST scope that helper constant to the test code rather than exposing it to the non-test binary build.
AND Mantle MUST NOT turn the constant into proof evidence or a source-built closure claim unless runtime proof receipts actually use it.

#### Scenario: Dormant surfaces use scoped allowances

GIVEN a provider/front-end/native-planner surface is intentionally compiled but not wired into the current native Mantle binary path
WHEN that surface would emit `dead_code` warnings during provider-backed topology execution
THEN Mantle MAY apply item- or module-scoped `dead_code` allowances for that dormant surface.
AND Mantle MUST NOT apply crate-wide unused-code suppression that would hide unrelated warning regressions.

#### Scenario: Provider proof frontier is rerun honestly

GIVEN the Mantle binary warning frontier has been addressed
WHEN the provider-backed fixed-point proof is rerun from current code
THEN Mantle MUST record whether fixed-point succeeds or the next deterministic blocker appears.
AND any blocked run MUST retain bounded non-claims instead of reporting provider fixed-point release artifact evidence.
