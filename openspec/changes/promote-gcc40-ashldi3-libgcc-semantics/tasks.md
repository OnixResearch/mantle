## Phase 1: Implementation

- [x] [serial] Document `_ashldi3` as the next semantic member in `bootstrap/gcc-4.0.ncl`. ✅ 1m (started: 2026-05-12T04:00:00Z → completed: 2026-05-12T04:01:00Z)
  Evidence: `bootstrap/gcc-4.0.ncl` comments now identify `_ashldi3` as the fourth bounded semantic promotion for 64-bit left shifts below 64.
- [ ] [serial] Replace the `_ashldi3` placeholder body while preserving deterministic ar(5) archive generation.
- [ ] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus `_ashldi3` semantic smoke.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate promote-gcc40-ashldi3-libgcc-semantics --strict` and record evidence before archive.
