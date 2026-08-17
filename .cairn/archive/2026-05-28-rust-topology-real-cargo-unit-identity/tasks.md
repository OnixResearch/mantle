# Tasks

## Spec

- [x] [serial] Add real selected Cargo unit identity requirement and design. r[rust_package_planning.native_real_unit_identity]

## Implementation

- [x] [serial] Replace package/crate/kind/mode producer keys with selected Cargo unit IDs from unit graph entries. r[rust_package_planning.native_real_unit_identity]
- [x] [serial] Preserve duplicate selected Cargo target units during native target graph planning. r[rust_package_planning.native_real_unit_identity]
- [x] [serial] Constrain deduplication to exact duplicate unit IDs only. r[rust_package_planning.native_real_unit_identity]
- [x] [serial] Filter package-only fallback candidates by package ID plus crate name. r[rust_package_planning.native_real_unit_identity]
- [x] [serial] Add focused positive and negative tests for production lowering, duplicate selected units, selected search closure, and ambiguous fallback. r[rust_package_planning.native_real_unit_identity]

## Verification

- [x] [serial] Run fmt, diff check, focused unit tests, native unit graph tests, dirty self-probe, and Cairn validation/gates. Evidence: `evidence/verification.md`. r[rust_package_planning.native_real_unit_identity]
