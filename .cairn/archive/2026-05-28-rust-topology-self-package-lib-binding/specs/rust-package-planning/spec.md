## ADDED Requirements

### Requirement: Native self-package lib dependencies stay target-specific

r[rust_package_planning.native_self_package_lib_binding] Native Rust unit graph planning MUST bind same-package library artifacts only to package targets that actually consume that library, and MUST NOT attach package-level self dependency artifacts to the package library unit.

#### Scenario: package bin keeps same-package lib artifact

GIVEN a package has supported `lib` and `bin` targets
AND the Cargo-selected unit graph records the bin consuming the same-package lib artifact
WHEN Mantle plans native target units
THEN the native bin unit MUST contain exactly the same-package lib dependency artifact with the lib crate name.

#### Scenario: package lib rejects self dependency artifact

GIVEN a package has supported `lib` and `bin` targets
AND selected package dependency artifacts include the bin's same-package lib edge
WHEN Mantle plans the native lib unit
THEN the native lib unit MUST NOT contain a dependency artifact for its own package.
