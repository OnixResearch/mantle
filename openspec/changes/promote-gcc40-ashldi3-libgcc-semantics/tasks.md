## Phase 1: Implementation

- [x] [serial] Document `_ashldi3` as the next semantic member in `bootstrap/gcc-4.0.ncl`. ✅ 1m (started: 2026-05-12T04:00:00Z → completed: 2026-05-12T04:01:00Z)
  Evidence: `bootstrap/gcc-4.0.ncl` comments now identify `_ashldi3` as the fourth bounded semantic promotion for 64-bit left shifts below 64.
- [x] [serial] Replace the `_ashldi3` placeholder body while preserving deterministic ar(5) archive generation. ✅ 1m (started: 2026-05-12T04:01:05Z → completed: 2026-05-12T04:02:00Z)
  Evidence: `_ashldi3` now emits `long long _ashldi3(long long value, int count) { return value << count; }`; `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` produced `gcc-4.0.4 script_bytes 56481 inputs 11`, `/bin/sh -n` on the generated builder passed, and `git diff --check` passed.
- [x] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus `_ashldi3` semantic smoke. ✅ 2m (started: 2026-05-12T04:02:10Z → completed: 2026-05-12T04:04:00Z)
  Evidence: `./target/debug/crunch build bootstrap/gcc-4.0.ncl --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned --no-substitute` produced `.crunch-drain/store/ajs071z0k5q5liiygisqcmnwfbwvvgm5-gcc-4.0.4`; artifact checks passed for `gcc`, `cc`, `cc1`, `crtbegin.o`, `crtend.o`, and `libgcc.a`; host `ar`/`nm` saw `_ashldi3`, `_lshrdi3`, `_muldi3`, and `_negdi2`; Nix clang-wrapper linked extracted `_ashldi3.o` into a semantic smoke proving zero shift, `1 << 7 = 128`, `0x1234 << 16 = 0x12340000`, `1 << 62`, and high-bit-producing `1 << 63`.

## Phase 2: Completion

- [x] [depends:implementation] Run `openspec validate promote-gcc40-ashldi3-libgcc-semantics --strict` and record evidence before archive. ✅ 1m (started: 2026-05-12T04:04:05Z → completed: 2026-05-12T04:04:20Z)
  Evidence: `openspec validate promote-gcc40-ashldi3-libgcc-semantics --strict` passed before archive.
