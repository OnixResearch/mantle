## ADDED Requirements

### Requirement: Native manifest links build-script env

r[rust_package_planning.native_manifest_links_env] Mantle MUST provide deterministic `CARGO_MANIFEST_LINKS` to native build scripts from the package manifest `links` field.

#### Scenario: Linked package receives links env

GIVEN a package manifest declares `[package].links = "ring_core_0_17_14"`
WHEN Mantle prepares a native build-script child environment
THEN the child environment MUST include `CARGO_MANIFEST_LINKS=ring_core_0_17_14`.

#### Scenario: Unlinked package receives empty links env

GIVEN a package manifest does not declare `[package].links`
WHEN Mantle prepares a native build-script child environment
THEN the child environment MUST include `CARGO_MANIFEST_LINKS` with an empty string value.

#### Scenario: Ring missing-links frontier moves

GIVEN topology execution currently reaches `ring`'s build script and panics while unwrapping `CARGO_MANIFEST_LINKS`
WHEN manifest links env parity is applied
THEN self-probe verification MUST show that the `CARGO_MANIFEST_LINKS` unwrap panic no longer stops topology.
AND any remaining blocker MUST be recorded with deterministic class and evidence.
