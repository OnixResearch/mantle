## Phase 1: Implementation

- [x] [serial] Document `_ashrdi3` as the next semantic member in `bootstrap/gcc-4.0.ncl`. ✅ 1m (started: 2026-05-12T04:04:30Z → completed: 2026-05-12T04:05:15Z)
  Evidence: `bootstrap/gcc-4.0.ncl` comments now identify `_ashrdi3` as the fifth bounded semantic promotion for signed 64-bit arithmetic right shifts below 64.
- [ ] [serial] Replace the `_ashrdi3` placeholder body while preserving deterministic ar(5) archive generation.
- [ ] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus `_ashrdi3` semantic smoke.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate promote-gcc40-ashrdi3-libgcc-semantics --strict` and record evidence before archive.
