## Phase 1: Exact blocker classification

- [x] [serial] I1 [covers=bootstrap_inventory.full_check_blocker_closure] Record the 115-finding baseline and split proof-bound markers from lexical false positives. r[bootstrap_inventory.full_check_blocker_closure]
- [x] [serial] I2 [covers=bootstrap_inventory.full_check_blocker_closure] Add whole-file V98 BLAKE3 classification for named paths and marker classes. r[bootstrap_inventory.full_check_blocker_closure]
- [x] [serial] I3 [covers=bootstrap_inventory.full_check_blocker_closure] Add narrow structural rules for negative bridge facts and bounded timeout controls. r[bootstrap_inventory.full_check_blocker_closure]
- [x] [serial] I4 [covers=bootstrap_inventory.full_check_blocker_closure] Add ADR and operator documentation for exact marker retirement. r[bootstrap_inventory.full_check_blocker_closure]

## Phase 2: Fixed-output repair

- [x] [serial] I5 [covers=bootstrap_inventory.full_check_blocker_closure] Rebuild SpaceWasm and the wasm-component toolchain locally, then repair only independently confirmed immutable hashes. r[bootstrap_inventory.full_check_blocker_closure]
  - Evidence: Crane vendoring removed the blocked API transport without changing the SpaceWasm revision, lock, toolchain, targets, or features. Fresh and `--rebuild` bundle runs pass. The wasm-component toolchain `--rebuild`, including pinned Octet, also passes.

## Phase 3: Verification

- [ ] [serial] V1 [covers=bootstrap_inventory.full_check_blocker_closure] [evidence=evidence/validation.md] Run positive clean-inventory checks and negative proof-byte, unknown-path, unknown-class, positive-bridge, and observed-timeout cases. r[bootstrap_inventory.full_check_blocker_closure]
- [ ] [serial] V2 [covers=bootstrap_inventory.full_check_blocker_closure] [evidence=evidence/validation.md] Run focused Nix builds, local-builder and ordinary full flake checks, formatting, diff checks, Cairn validation, Tracey coverage, and all three gates. r[bootstrap_inventory.full_check_blocker_closure]
- [ ] [serial] V3 [covers=bootstrap_inventory.full_check_blocker_closure] [evidence=evidence/validation.md] Preserve exact later blockers or confirm a clean full check, then sync and archive from committed source. r[bootstrap_inventory.full_check_blocker_closure]
