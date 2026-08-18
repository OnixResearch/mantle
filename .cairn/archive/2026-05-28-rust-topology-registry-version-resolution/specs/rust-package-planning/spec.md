## ADDED Requirements

### Requirement: Native registry dependency version resolution

r[rust_package_planning.native_registry_dependency_version_resolution] Mantle MUST prefer the vendored registry source identified by a dependency's declared version when same-name vendored registry packages exist.

#### Scenario: Exact version selects matching source

GIVEN vendored registry sources include `thiserror-impl` versions `1.0.69` and `2.0.18`
WHEN a dependency declares `thiserror-impl = "=2.0.18"`
THEN native package planning MUST select the `thiserror-impl@2.0.18` manifest path.

#### Scenario: Missing exact version fails closed

GIVEN vendored registry sources include multiple versions with the same package name
WHEN a dependency declares an exact version that is absent from vendored sources
THEN native package planning MUST report a deterministic missing-version blocker.

#### Scenario: Thiserror private API frontier moves

GIVEN topology execution currently links `thiserror@2.0.18` with the `thiserror-impl@1.0.69` proc macro
WHEN version-aware registry dependency resolution is applied
THEN self-probe verification MUST show `thiserror@2.0.18` consumes `thiserror-impl@2.0.18`.
AND any remaining blocker MUST be recorded with deterministic class and evidence.
