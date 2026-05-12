## Phase 1: Implementation

- [x] [serial] Document `_lshrdi3` as the next semantic member in `bootstrap/gcc-4.0.ncl`. ✅ 1m (started: 2026-05-12T03:47:03Z → completed: 2026-05-12T03:48:06Z)
  Evidence: `bootstrap/gcc-4.0.ncl` comments now identify `_lshrdi3` as the third bounded semantic promotion for unsigned 64-bit logical right shifts below 64.
- [ ] [serial] Replace the `_lshrdi3` placeholder body while preserving deterministic ar(5) archive generation.
- [ ] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus `_lshrdi3` semantic smoke.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate promote-gcc40-lshrdi3-libgcc-semantics --strict` and record evidence before archive.
