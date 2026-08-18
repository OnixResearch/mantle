# Rust Package Planning Specification Delta

## ADDED Requirements

### Requirement: Rust topology external linker mode

r[rust_package_planning.rust_topology_external_linker] Mantle MUST avoid host-rustup self-contained linker wrapper failures when selected Rust unit args do not explicitly request them.

#### Scenario: Runtime args disable self-contained linker by default

GIVEN a selected Rust unit has no `link-self-contained` rustc codegen option
WHEN Mantle executes that unit through the Rust topology rail
THEN Mantle MUST add `-C link-self-contained=no` to the runtime rustc invocation.
AND Mantle MUST keep the selected unit's original reviewable args otherwise unchanged.

#### Scenario: Explicit self-contained linker choice is preserved

GIVEN a selected Rust unit already contains a `link-self-contained` rustc codegen option
WHEN Mantle derives runtime rustc args
THEN Mantle MUST NOT add another `link-self-contained` option.
AND Mantle MUST preserve the selected unit's explicit value.

#### Scenario: Stale rustup linker wrapper blocker moves

GIVEN topology execution currently fails because rustup's `gcc-ld/ld.lld` wrapper references a missing Nix store `ld-wrapper.sh`
WHEN external linker mode is applied
THEN self-probe verification MUST show that this specific missing `ld-wrapper.sh` failure no longer blocks the first topology unit.
AND any remaining blocker MUST be recorded with deterministic class and evidence.
