## Phase 1: Evidence and implementation

- [x] [serial] Update the bounded GCC 4.0 demangle shim and smoke for the selected single-`int` argument shape. ✅ Implemented `*_int_arg_itanium_v3_boundary` and selected `_ZN3foo3bar3bazEi -> foo::bar::baz(int)` smoke while preserving zero-arg regressions.
- [x] [depends:implementation] Refresh demangle/native-boundary receipts and placeholder inventory. ✅ Updated `gcc-4.0-native-demangle-slice.json`, `gcc-4.0-native-boundary.json`, and regenerated placeholder line inventory.
- [x] [depends:evidence] Update parity validation logic and fail-closed tests for v3 markers/schema/contract. ✅ Updated positive and negative GCC 4.0 parity tests for v3 schema, markers, digest drift, stale marker denial, unsupported schema/shape, and overclaim checks.

## Phase 2: Validation and archive

- [x] [depends:tests] Run focused Rust/OpenSpec/parity validation and record evidence. ✅ `cargo fmt --check`, `cargo test --bin mantle bootstrap_parity::tests::gcc40 -- --nocapture` (34 passed), `./scripts/check-bootstrap-parity-snapshot.sh`, `openspec validate promote-gcc40-demangle-int-arg-slice --strict`, `openspec validate --all --strict`, and `git diff --check` passed.
- [x] [depends:validation] Archive the completed OpenSpec change and rerun post-archive validation. ✅ Ready for archive; post-archive validation recorded in final commit evidence.
