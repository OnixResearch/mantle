# Tasks

## Phase 1: Policy and plan identity

- [x] [depends:audit-foreign-realization-provenance] I1 Add `preserve-cache-paths-v1` policy admission with unchanged-prefix and cache-only constraints. r[foreign_derivation_import.cache_only_preserved_paths]
- [x] [serial] I2 Compile exact identity output and source maps while retaining Mantle derivation and profile identity. r[foreign_derivation_import.cache_only_preserved_paths]
- [x] [serial] I3 Bind `cache-only-preserve-v1` into executable-plan identity and reject malformed route combinations. r[foreign_derivation_import.cache_only_preserved_paths]
- [x] [parallel] I4 Add positive and negative policy, compiler, plan, collision, and profile tests. r[foreign_derivation_import.cache_only_preserved_paths]

## Phase 2: Signed runtime closure hydration

- [x] [serial] I5 Add a pure bounded cache-closure work planner and deterministic hydration facts. r[foreign_derivation_import.cache_only_runtime_closure]
- [x] [serial] I6 Hydrate selected roots and runtime references through signed NARInfo, NAR, PathInfo, and castore admission. r[foreign_derivation_import.cache_only_runtime_closure]
- [x] [serial] I7 Reject source records, offline mode, remote execution, cache misses, invalid signatures, mismatched paths, and local build fallback. r[foreign_derivation_import.cache_only_runtime_closure]
- [x] [serial] I8 Run the ordinary registry, scheduler, worker, and store only after the selected runtime closure is complete. r[foreign_derivation_import.cache_only_runtime_closure]
- [x] [parallel] I9 Add local HTTP cache fixtures for positive closure hydration, missing members, bad signatures, NAR mismatch, limits, rerun reuse, and no-build enforcement. r[foreign_derivation_import.cache_only_runtime_closure]

## Phase 3: Receipts, proof, and operator contract

- [x] [serial] I10 Extend realization receipts with route-bound closure hydration facts and not-required unit dispositions. r[foreign_derivation_import.live_nixpkgs_realization_proof]
- [x] [parallel] I11 Update trust, operator, machine-contract, and README documentation. r[foreign_derivation_import.live_nixpkgs_realization_proof]
- [x] [serial] I12 Capture a fresh host-Nix `nixpkgs#hello` export, then consume it with a PATH that contains no Nix frontend command. r[foreign_derivation_import.live_nixpkgs_realization_proof]

## Phase 4: Verification and lifecycle

- [x] [serial] V1 Run focused policy, compiler, realization, store, receipt, provenance, and CLI tests. Record exact commands and statuses. r[foreign_derivation_import.cache_only_runtime_closure]
- [x] [serial] V2 Run live `nixpkgs#hello` realization, exact rerun reuse, provenance audit, and fresh-store receipt-bound cache hydration. Record identities and non-claims. r[foreign_derivation_import.live_nixpkgs_realization_proof]
- [x] [serial] V3 Run formatting, workspace check, focused Clippy, Nickel checks, trust-model guard, Cairn validation, all gates, and Tracey coverage. r[foreign_derivation_import.live_nixpkgs_realization_proof]
