## Phase 1: Bounded long demangle promotion

- [x] [serial] Update bounded GCC 4.0 libiberty demangle shim and smoke for `_ZN3foo3bar3bazEl -> foo::bar::baz(long)` while preserving prior zero-arg, `int`, and `char` regressions.
- [x] [serial] Update native demangle and boundary evidence receipts plus placeholder inventory.
- [x] [serial] Update parity validation tests and stale-marker denial for v5 long markers and prior v4 markers.
- [x] [serial] Run focused Rust/OpenSpec/parity validation.
- [x] [serial] Archive the OpenSpec change after all tasks pass and repair canonical spec drift if archive drops prior scenarios.
