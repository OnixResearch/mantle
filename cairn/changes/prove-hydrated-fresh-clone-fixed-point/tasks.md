## Phase 1: Source authority and fail-closed execution

- [ ] [serial] I1 Add a distinct full-proof source profile and pure closure/profile validators while preserving the exact three-record fresh-clone hydration contract. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]
- [ ] [serial] I2 Add explicit connected materialization of missing evaluated fixed-fetch records through Mantle's fixed-output verifier, with bounded payload and deterministic failure behavior. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]
- [ ] [serial] I3 Add fetch-service/pipeline/self-build enforcement so unmatched builtin fetches fail before network acquisition and successful reports bind source-state identity and zero live fetches. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]

## Phase 2: Proof harness and contracts

- [ ] [serial] I4 Extend the proof helper/test to seed fresh source-only state for both stages, enforce offline source policy, and preserve contracted hydration/source-policy evidence. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]
- [ ] [serial] I5 Add ADR 0032, operator/proof documentation, schemas, fixtures, and machine-contract coverage with explicit claim boundaries. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]

## Phase 3: Verification and lifecycle

- [ ] [serial] V1 Preserve pre-change focused source-bundle, self-build, and proof-harness baseline results. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]
- [ ] [serial] V2 Add positive closure/materialization/offline-stage tests and negative missing, duplicate, stale, tampered, URL/revision mismatch, unsupported kind, and attempted live-fetch tests. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]
- [ ] [serial] V3 Produce one real full-proof bundle, hydrate a Git clone with initially empty Cargo/source/proof state, run locked offline Cargo metadata, and execute the complete stage0 → stage1 → stage2 proof with live source acquisition forbidden. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]
- [ ] [serial] V4 Run Rustfmt, strict first-party Clippy, Tiger Style, dependency policy, machine contracts/docs, first-party quality, Nix evaluation, Cairn gates, and Tracey coverage. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]
- [ ] [serial] V5 Commit implementation before archiving; sync and inspect the accepted requirement; archive exact post-state receipts; commit and push `main`. r[bootstrap_inventory.hydrated_fresh_clone_fixed_point]
