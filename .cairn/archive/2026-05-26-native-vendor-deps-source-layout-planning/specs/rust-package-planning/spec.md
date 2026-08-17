# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_vendor_deps_source_layout_planning]

Mantle MUST bind checkout-local `vendor-deps/` registry source layouts as explicit native planning material.

#### Scenario: conventional vendor-deps source root is discovered

- GIVEN a Rust workspace checkout contains `vendor-deps/`
- WHEN native registry source planning runs
- THEN the planner MUST treat that directory as declared local registry source material
- AND it MUST NOT require `$CARGO_HOME`, network access, or an ambient Cargo cache for those sources.

#### Scenario: unversioned vendor directory binds registry package identity

- GIVEN `Cargo.lock` contains a registry package with checksum evidence
- AND the checkout contains `vendor-deps/<crate>/Cargo.toml`
- WHEN native registry source planning binds the package
- THEN it MUST use the local manifest and `.cargo-checksum.json`
- AND it MUST emit source digest evidence for the local tree.

#### Scenario: Cargo oracle comparison uses package identity

- GIVEN Cargo metadata reports the same registry package from a Cargo cache manifest path
- AND native planning bound the package from `vendor-deps/<crate>/Cargo.toml`
- WHEN native package/target comparison runs
- THEN it MUST match by package ID before comparing manifest paths
- AND it MUST NOT emit `cargo-oracle-missing-package` for that package solely because the local vendor path differs from Cargo's cache path.
