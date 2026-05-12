## Phase 1: Implementation

- [x] [serial] Document `_muldi3` as the selected second semantic member in `bootstrap/gcc-4.0.ncl`. ✅ 1m (started: 2026-05-12T03:39:45Z → completed: 2026-05-12T03:40:04Z)
  Evidence: `bootstrap/gcc-4.0.ncl` comments now identify `_muldi3` as the second bounded semantic promotion for non-overflowing signed 64-bit multiplication; `git diff --check` passed.
- [ ] [serial] Replace the `_muldi3` placeholder body while preserving deterministic ar(5) archive generation.
- [ ] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus `_muldi3` semantic smoke.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate promote-gcc40-muldi3-libgcc-semantics --strict` and record evidence before archive.
