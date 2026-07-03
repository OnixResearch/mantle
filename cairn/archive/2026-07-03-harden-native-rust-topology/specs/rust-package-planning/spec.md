## ADDED Requirements

### Requirement: Native Rust topology hardening

r[rust_package_planning.native_topology_hardening] Mantle MUST provide deterministic diagnostics, bounded replay evidence, and positive plus negative coverage for native Rust topology behavior that affects Cargo-free builds.

#### Scenario: topology diagnostics name stable identities

GIVEN native Rust topology planning or execution rejects, blocks, or cannot order a unit
WHEN Mantle reports the diagnostic
THEN the diagnostic MUST include stable native unit identity, package identity, execution role, selected triple, target kind, artifact role, and blocker class when available.
AND it MUST NOT require Cargo unit indices as the only way to understand the failure.

#### Scenario: replay evidence is bounded

GIVEN a native Rust unit fails or blocks during topology execution
WHEN Mantle records replay evidence
THEN the receipt MUST contain the deterministic facts needed to explain or replay the unit boundary.
AND large or source-derived inputs MUST be represented by BLAKE3 digests or bounded summaries unless the full inline value is explicitly required.

#### Scenario: role and metadata mismatches fail before rustc

GIVEN a native Rust unit consumes an artifact with the wrong role, selected triple, source package, rustc metadata, or toolchain policy digest
WHEN Mantle validates the topology artifact graph
THEN validation MUST fail before invoking rustc.
AND the diagnostic MUST name the mismatched field and producer/consumer boundary.

#### Scenario: supported host-unit edges remain covered

GIVEN a Rust workspace uses build-script dependencies, linked metadata, proc macros, selected target features, or source-root host/target splits
WHEN focused native topology fixtures run
THEN Mantle MUST either build the supported topology edge or report an explicit unsupported blocker.
AND it MUST NOT silently fall back to ambient Cargo planning.
