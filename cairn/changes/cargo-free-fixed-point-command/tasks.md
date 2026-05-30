# Tasks

## Spec

- [ ] [serial] Add the first-class Cargo-free fixed-point command delta. r[rust_package_planning.cargo_free_fixed_point_command]

## Implementation

- [ ] [serial] Add `mantle self-build --cargo-free --fixed-point --out <bundle-dir>` CLI parsing and conflict validation. r[rust_package_planning.cargo_free_fixed_point_command]
- [ ] [serial] Extract a pure fixed-point planning core for stage paths, guard paths, execution roots, evidence paths, and command descriptors. r[rust_package_planning.cargo_free_fixed_point_command]
- [ ] [serial] Move stage execution, receipt-derived Mantle binary selection, smoke checks, BLAKE3 comparison, and summary writing behind the first-class command. r[rust_package_planning.cargo_free_fixed_point_command]
- [ ] [serial] Add command-owned rustc/linker compatibility probing or recorded normalization so callers do not need an external `/tmp` rustc wrapper. r[rust_package_planning.cargo_free_fixed_point_command]

## Verification

- [ ] [serial] Add positive CLI tests for a tiny fixture where stage1/stage2 fixed-point mode succeeds. r[rust_package_planning.cargo_free_fixed_point_command]
- [ ] [serial] Add negative CLI tests for Cargo guard invocation, toolchain incompatibility, inside-source output directories, and stage digest mismatch/blocker evidence. r[rust_package_planning.cargo_free_fixed_point_command]
- [ ] [serial] Run a real Mantle command equivalent to `mantle self-build --cargo-free --fixed-point --out /tmp/<bundle>` and record status, stage digests, unit counts, and Cargo marker state. r[rust_package_planning.cargo_free_fixed_point_command]
- [ ] [serial] Run `cairn validate --root .`. r[rust_package_planning.cargo_free_fixed_point_command]
- [ ] [serial] Run proposal, design, and tasks gates for this change. r[rust_package_planning.cargo_free_fixed_point_command]
