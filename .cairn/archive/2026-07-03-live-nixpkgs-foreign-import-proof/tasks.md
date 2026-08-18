## Implementation

- [x] [serial] I1 Capture live host-Nix `nixpkgs#hello` drv resolution and recursive derivation JSON export evidence. r[foreign_derivation_import.live_nixpkgs_export_to_plan_proof]
- [x] [serial] I2 Run `mantle foreign-import produce-nix` on the live derivation JSON and retain bounded graph/index artifacts in change evidence. r[foreign_derivation_import.live_nixpkgs_export_to_plan_proof]
- [x] [serial] I3 Validate and plan the produced artifacts with consumption PATH stripped of `nix`/`nix-store`, and record explicit admission/planning non-claims. r[foreign_derivation_import.live_nixpkgs_export_to_plan_proof]
- [x] [serial] I4 Update operator docs to point to the live proof boundary without claiming substitution or rebuild support. r[foreign_derivation_import.live_nixpkgs_export_to_plan_proof]

## Verification

- [x] [serial] V1 Positive: evidence shows a real `nixpkgs#hello` drv path, successful derivation JSON export, successful `produce-nix`, and successful validate/plan with fake PATH. r[foreign_derivation_import.live_nixpkgs_export_to_plan_proof]
- [x] [serial] V2 Negative: evidence shows no-Nix consumption by using a fake PATH and checking the resulting plan has no forbidden process invocations. r[foreign_derivation_import.live_nixpkgs_export_to_plan_proof]
- [x] [serial] V3 Run trust-model guard, Cairn validation, and Cairn proposal/design/tasks gates. r[foreign_derivation_import.live_nixpkgs_export_to_plan_proof]
