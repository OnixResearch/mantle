## Implementation

- [x] [serial] I1 Add the `--drv-dir` reachable-closure requirement to the foreign derivation import spec delta. r[foreign_derivation_import.drv_dir_closure_producer]
- [x] [serial] I2 Implement a pure root-reachable Nix closure selection helper that fails closed on missing input derivations. r[foreign_derivation_import.drv_dir_closure_producer]
- [x] [serial] I3 Add CLI shell support for `foreign-import produce-nix --drv-dir <dir>` without mixing input modes or invoking foreign frontend commands. r[foreign_derivation_import.drv_dir_closure_producer]
- [x] [serial] I4 Add checked positive and negative tests for directory closure import. r[foreign_derivation_import.drv_dir_closure_producer]

## Verification

- [x] [serial] V1 Positive: produce artifacts from a checked-in `.drv` directory with fake `PATH`, validate, plan, and prove an unrelated `.drv` in the directory is not emitted for the selected root. r[foreign_derivation_import.drv_dir_closure_producer]
- [x] [serial] V2 Negative: omit a reachable input `.drv` from the directory and assert deterministic failure with no graph/index artifacts. r[foreign_derivation_import.drv_dir_closure_producer]
- [x] [serial] V3 Run focused foreign import tests, formatting, `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[foreign_derivation_import.drv_dir_closure_producer]
