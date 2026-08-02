# Tasks

## Phase 1: Producer identity and export

- [x] [serial] I1 Pin a live GuixPkgs revision and record its Guix and `guix-transfer` source identities. r[foreign_derivation_import.live_guixpkgs_export_realization]
- [x] [serial] I2 Export the recursive `hello.unwrapped` derivation graph and validate its selected root. r[foreign_derivation_import.live_guixpkgs_export_realization]
- [x] [serial] I3 Realize, sign, and export the selected translated runtime closure under a dedicated proof key. r[foreign_derivation_import.live_guixpkgs_export_realization]
- [x] [serial] I4 Produce a cache-only preserved-path plan and an empty source bundle. r[foreign_derivation_import.live_guixpkgs_export_realization]

## Phase 2: Guix-free realization proof

- [x] [serial] I5 Realize the signed export-cache runtime closure through Mantle with Nix and Guix absent from `PATH`. r[foreign_derivation_import.live_guixpkgs_export_realization]
- [x] [serial] I6 Prove exact reuse and receipt-selected fresh-store hydration without builder fallback. r[foreign_derivation_import.live_guixpkgs_export_realization]
- [x] [serial] I7 Run bounded castore provenance audit and retain its exact findings. r[foreign_derivation_import.live_guixpkgs_export_realization]
- [x] [parallel] I8 Capture wrong-key, limit, receipt-tamper, and missing-member failures. r[foreign_derivation_import.live_guixpkgs_export_realization]

## Phase 3: Contract and documentation

- [x] [serial] I9 Add producer metadata, transcript, summary, implementation, and verification evidence. r[foreign_derivation_import.live_guixpkgs_export_realization]
- [x] [parallel] I10 Update README, trust-model, operator, and machine-artifact documentation. r[foreign_derivation_import.live_guixpkgs_export_realization]
- [x] [parallel] I11 Record the adopted GuixPkgs, `guix-transfer`, and Guix-by-Nix references. r[foreign_derivation_import.live_guixpkgs_export_realization]

## Phase 4: Verification and lifecycle

- [x] [serial] V1 Run focused import, cache closure, realization, receipt, audit, and CLI tests. r[foreign_derivation_import.live_guixpkgs_export_realization]
- [x] [serial] V2 Run formatting, workspace check, focused Clippy, Nickel checks, trust-model guard, and machine-contract checks. r[foreign_derivation_import.live_guixpkgs_export_realization]
- [ ] [serial] V3 Run Cairn validation, gates, sync, archive, and publication. r[foreign_derivation_import.live_guixpkgs_export_realization]
