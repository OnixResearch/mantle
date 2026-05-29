# Tasks

## Spec

- [x] [serial] Add native unit graph construction requirement and design. r[rust_package_planning.native_unit_graph]

## Implementation

- [ ] [serial] Construct target lib/bin units from native package and feature facts. r[rust_package_planning.native_unit_graph]
- [ ] [serial] Construct proc-macro and custom-build host units from native package and feature facts. r[rust_package_planning.native_unit_graph]
- [ ] [serial] Build typed dependency, host artifact, and build-script metadata edges. r[rust_package_planning.native_unit_graph]
- [ ] [serial] Define stable native unit IDs independent of Cargo unit indices. r[rust_package_planning.native_unit_graph]
- [ ] [serial] Emit fail-closed blockers for unsupported graph shapes. r[rust_package_planning.native_unit_graph]

## Verification

- [ ] [serial] Run native graph unit tests, Cargo oracle parity tests, topology execution smoke tests, and Cairn validation. r[rust_package_planning.native_unit_graph]
