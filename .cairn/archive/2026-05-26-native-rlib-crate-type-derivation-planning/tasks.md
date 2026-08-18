# Tasks: Native rlib crate-type derivation planning

- [x] [serial] Accept `rlib` as library-compatible in Cargo oracle package/target comparison.
- [x] [serial] Select unit target kind arrays containing `rlib` as `target_kind = "lib"`.
- [x] [serial] Add CLI coverage for a mixed `["rlib", "cdylib"]` library target.
- [x] [serial] Verify Mantle's current unit derivation graph no longer blocks on the `irpc` mixed rlib/cdylib target.
- [x] [serial] Sync accepted spec and archive the change after verification.
