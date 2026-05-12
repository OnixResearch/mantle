## Phase 1: Implementation

- [x] [serial] Document `_muldi3` as the selected second semantic member in `bootstrap/gcc-4.0.ncl`. ✅ 1m (started: 2026-05-12T03:39:45Z → completed: 2026-05-12T03:40:04Z)
  Evidence: `bootstrap/gcc-4.0.ncl` comments now identify `_muldi3` as the second bounded semantic promotion for non-overflowing signed 64-bit multiplication; `git diff --check` passed.
- [x] [serial] Replace the `_muldi3` placeholder body while preserving deterministic ar(5) archive generation. ✅ 1m (started: 2026-05-12T03:40:24Z → completed: 2026-05-12T03:41:20Z)
  Evidence: `_muldi3` now emits `long long _muldi3(long long lhs, long long rhs) { return lhs * rhs; }`; `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` produced `gcc-4.0.4 script_bytes 55910 inputs 11`, `/bin/sh -n` on the generated builder passed, and `git diff --check` passed.
- [ ] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus `_muldi3` semantic smoke.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate promote-gcc40-muldi3-libgcc-semantics --strict` and record evidence before archive.
