## Phase 1: Implementation

- [x] [serial] Document `_lshrdi3` as the next semantic member in `bootstrap/gcc-4.0.ncl`. ✅ 1m (started: 2026-05-12T03:47:03Z → completed: 2026-05-12T03:48:06Z)
  Evidence: `bootstrap/gcc-4.0.ncl` comments now identify `_lshrdi3` as the third bounded semantic promotion for unsigned 64-bit logical right shifts below 64.
- [x] [serial] Replace the `_lshrdi3` placeholder body while preserving deterministic ar(5) archive generation. ✅ 1m (started: 2026-05-12T03:48:17Z → completed: 2026-05-12T03:49:22Z)
  Evidence: `_lshrdi3` now emits `unsigned long long _lshrdi3(unsigned long long value, int count) { return value >> count; }`; `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` produced `gcc-4.0.4 script_bytes 56217 inputs 11`, `/bin/sh -n` on the generated builder passed, and `git diff --check` passed.
- [ ] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus `_lshrdi3` semantic smoke.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate promote-gcc40-lshrdi3-libgcc-semantics --strict` and record evidence before archive.
