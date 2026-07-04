# Rust Package Planning Specification

## Purpose

Defines additional requirements for repairing the current Cargo-free fixed-point proof frontier without weakening source-material integrity checks.

## Requirements

### Requirement: Vendor material checksum repair remains fail-closed

r[rust_package_planning.vendor_material_checksum_repair] Cargo-free native Rust planning MUST require vendored registry source material, Cargo checksum metadata, and `Cargo.lock` checksum expectations to agree before a source package is admitted into the native source closure.

#### Scenario: Repaired vendored package matches lock material

GIVEN vendored material for a registry package that appears in `Cargo.lock`
WHEN native registry source planning computes the source-material digest
THEN the computed material MUST match the lockfile/checksum metadata expected by the planner
AND planning MUST admit the package without fabricating or ignoring checksum facts.

#### Scenario: Drift remains a hard blocker

GIVEN vendored material differs from the checksum material declared for a registry package
WHEN native registry source planning validates the package
THEN planning MUST fail closed with a deterministic vendor-checksum diagnostic
AND it MUST NOT continue by using ambient Cargo, network refetching, or unchecked source contents.

### Requirement: Vendor source material drift diagnostics are reviewable

r[rust_package_planning.vendor_source_material_drift_diagnostics] Cargo-free source-material diagnostics MUST identify the affected package, blocker class, planning path, and digest evidence needed to reproduce a vendor checksum mismatch.

#### Scenario: Checksum mismatch explains the blocked package

GIVEN a registry package fails vendored checksum validation
WHEN the Cargo-free blocker classifier summarizes the receipt
THEN the diagnostic MUST name the package identity, blocker class, and nested planning path
AND the diagnostic SHOULD include bounded digest evidence rather than unbounded source listings.

### Requirement: Cargo-free fixed-point frontier is rerun after repair

r[rust_package_planning.cargo_free_fixed_point_frontier_rerun] After repairing a known source-material checksum frontier, Mantle MUST rerun the Cargo-free fixed-point proof or record why the rerun was blocked, and the resulting evidence MUST state success, blocked status, or environmental failure explicitly.

#### Scenario: Fixed-point proof succeeds after repair

GIVEN source material validation no longer blocks stage1 planning
WHEN the Cargo-free fixed-point proof completes
THEN the evidence MUST record stage binary digests, fixed-point status, receipt digests, and the command transcript
AND it MUST state that success applies only to the proven proof mode.

#### Scenario: Frontier moves after repair

GIVEN source material validation no longer reports the repaired package mismatch
WHEN the fixed-point proof fails or blocks on a later frontier
THEN the evidence MUST record the next blocker class and location
AND it MUST NOT claim fixed-point success.
