## Phase 1: Rebuild proof

- [x] [serial] Validate the OpenSpec change package before running the rebuild. ✅ (completed: 2026-05-14T15:01:55Z)
- [x] [serial] Rebuild `packages/clankers/clankers.ncl` in a fresh `.crunch-drain/clankers-root-repro-*` store. ✅ (started: 2026-05-14T15:04:18Z → completed: 2026-05-14T15:15:46Z)
  - Evidence: `.crunch-drain/clankers-root-repro-1778771058/exit-status` recorded `0`.
  - Rebuilt output: `.crunch-drain/clankers-root-repro-1778771058/store/xphfrsbmwr23rixlkbqs0bp93lfmkr41-clankers-0.1.0/bin/clankers`.
- [x] [depends:rebuild] Compute BLAKE3 for the rebuilt `$out/bin/clankers` and compare it to `e1e8e1c36e0979a2534bcb8c394d4c360068985b0700b0e73ee32f1bd23917ff`. ✅ (completed: 2026-05-14T15:16Z)
  - Original binary BLAKE3: `e1e8e1c36e0979a2534bcb8c394d4c360068985b0700b0e73ee32f1bd23917ff`.
  - Rebuilt binary BLAKE3: `82e569fcb115baf84efba01930b30085f0556d6ec9c3bc7c5ef45379e6aa5863`.
  - Verdict: `mismatch`; the final proof hash was not updated.
  - First differing byte offset: `86156127`; bounded inspection shows an embedded Cargo build-script output path changed from `cranelift-codegen-6a22cb1602628387/out/inst_builder.rs` to `cranelift-codegen-459ff1534d3e4b0b/out/inst_builder.rs`.
- [x] [depends:compare] Record a machine-readable reproducibility receipt with command, paths, digests, and verdict. ✅ (completed: 2026-05-14T15:17Z)
  - Receipt: `packages/clankers/clankers-root-rebuild-reproducibility.json`.
- [x] [depends:receipt] Validate JSON, derivation eval, and OpenSpec after evidence is recorded. ✅ (completed: 2026-05-14T15:17Z)
