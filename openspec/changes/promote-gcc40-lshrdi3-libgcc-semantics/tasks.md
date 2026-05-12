## Phase 1: Implementation

- [x] [serial] Document `_lshrdi3` as the next semantic member in `bootstrap/gcc-4.0.ncl`. ✅ 1m (started: 2026-05-12T03:47:03Z → completed: 2026-05-12T03:48:06Z)
  Evidence: `bootstrap/gcc-4.0.ncl` comments now identify `_lshrdi3` as the third bounded semantic promotion for unsigned 64-bit logical right shifts below 64.
- [x] [serial] Replace the `_lshrdi3` placeholder body while preserving deterministic ar(5) archive generation. ✅ 1m (started: 2026-05-12T03:48:17Z → completed: 2026-05-12T03:49:22Z)
  Evidence: `_lshrdi3` now emits `unsigned long long _lshrdi3(unsigned long long value, int count) { return value >> count; }`; `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` produced `gcc-4.0.4 script_bytes 56217 inputs 11`, `/bin/sh -n` on the generated builder passed, and `git diff --check` passed.
- [x] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus `_lshrdi3` semantic smoke. ✅ 2m (started: 2026-05-12T03:49:39Z → completed: 2026-05-12T03:51:16Z)
  Evidence: `./target/debug/crunch build bootstrap/gcc-4.0.ncl --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned --no-substitute` produced `.crunch-drain/store/6bbfc5vl26vjcg2smz19gcs7vrpir7al-gcc-4.0.4`; artifact checks passed for `gcc`, `cc`, `cc1`, `crtbegin.o`, `crtend.o`, and `libgcc.a`; host `ar`/`nm` saw `_lshrdi3`, `_muldi3`, and `_negdi2`; Nix clang-wrapper linked extracted `_lshrdi3.o` into a semantic smoke proving `0x8000000000000000 >> 63 = 1`, `0x8000000000000000 >> 60 = 8`, `0xf000000000000000 >> 4 = 0x0f00000000000000`, zero-shift identity, and all-ones `>> 32 = 0xffffffff`.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate promote-gcc40-lshrdi3-libgcc-semantics --strict` and record evidence before archive.
