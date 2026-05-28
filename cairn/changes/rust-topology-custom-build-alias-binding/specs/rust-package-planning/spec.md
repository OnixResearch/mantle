## ADDED Requirements

### Requirement: Native custom-build host artifacts bind Cargo build-script aliases

r[rust_package_planning.native_custom_build_alias_binding] Native topology execution MUST bind same-package custom-build host artifacts to Cargo build-script dependency aliases `build_script_main` and `build_script_build` before resolving normal target dependency artifacts.

#### Scenario: build_script_main placeholder is satisfied by custom-build host artifact

GIVEN a target unit consumes its same-package custom-build host artifact
AND the target unit has a dependency placeholder named `build_script_main` for that same package
WHEN host artifacts are bound for execution
THEN the dependency placeholder MUST be rewritten to the produced custom-build executable path
AND execution MUST NOT wait for a same-package target artifact that cannot exist.
