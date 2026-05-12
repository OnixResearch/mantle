## Phase 1: Implementation

- [x] [serial] Document `_ashrdi3` as the next semantic member in `bootstrap/gcc-4.0.ncl`. ✅ 1m (started: 2026-05-12T04:04:30Z → completed: 2026-05-12T04:05:15Z)
  Evidence: `bootstrap/gcc-4.0.ncl` comments now identify `_ashrdi3` as the fifth bounded semantic promotion for signed 64-bit arithmetic right shifts below 64.
- [x] [serial] Replace the `_ashrdi3` placeholder body while preserving deterministic ar(5) archive generation. ✅ 1m (started: 2026-05-12T04:05:20Z → completed: 2026-05-12T04:06:10Z)
  Evidence: `_ashrdi3` now emits `long long _ashrdi3(long long value, int count) { return value >> count; }`; `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` produced `gcc-4.0.4 script_bytes 56779 inputs 11`, `/bin/sh -n` on the generated builder passed, and `git diff --check` passed.
- [ ] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus `_ashrdi3` semantic smoke.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate promote-gcc40-ashrdi3-libgcc-semantics --strict` and record evidence before archive.
