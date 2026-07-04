## Implementation

- [x] [serial] I1 Add the direct `.drv` producer requirement to the foreign derivation import spec delta. r[foreign_derivation_import.direct_drv_producer]
- [x] [serial] I2 Implement a pure conversion helper from parsed Nix derivation facts to the existing Nix closure lowering input without filesystem, process, network, environment, clock, or store access. r[foreign_derivation_import.direct_drv_producer]
- [x] [serial] I3 Add CLI shell support for explicit `--drv <logical=path>` inputs on `foreign-import produce-nix`, keeping all file reads outside the core. r[foreign_derivation_import.direct_drv_producer]
- [x] [serial] I4 Add checked positive and negative tests for direct `.drv` artifact production without host Nix commands. r[foreign_derivation_import.direct_drv_producer]

## Verification

- [x] [serial] V1 Positive: produce artifacts from checked-in `.drv` fixture files with fake `PATH`, then validate and plan the result. r[foreign_derivation_import.direct_drv_producer]
- [x] [serial] V2 Negative: malformed `.drv` input fails with a deterministic diagnostic and no graph/index artifacts. r[foreign_derivation_import.direct_drv_producer]
- [x] [serial] V3 Run focused foreign import tests, formatting, `git diff --check`, Cairn validation, and Cairn proposal/design/tasks gates. r[foreign_derivation_import.direct_drv_producer]
