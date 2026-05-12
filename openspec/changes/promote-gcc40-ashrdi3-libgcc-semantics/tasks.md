## Phase 1: Implementation

- [x] [serial] Document `_ashrdi3` as the next semantic member in `bootstrap/gcc-4.0.ncl`. ✅ 1m (started: 2026-05-12T04:04:30Z → completed: 2026-05-12T04:05:15Z)
  Evidence: `bootstrap/gcc-4.0.ncl` comments now identify `_ashrdi3` as the fifth bounded semantic promotion for signed 64-bit arithmetic right shifts below 64.
- [x] [serial] Replace the `_ashrdi3` placeholder body while preserving deterministic ar(5) archive generation. ✅ 1m (started: 2026-05-12T04:05:20Z → completed: 2026-05-12T04:06:10Z)
  Evidence: `_ashrdi3` now emits `long long _ashrdi3(long long value, int count) { return value >> count; }`; `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` produced `gcc-4.0.4 script_bytes 56779 inputs 11`, `/bin/sh -n` on the generated builder passed, and `git diff --check` passed.
- [x] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus `_ashrdi3` semantic smoke. ✅ 2m (started: 2026-05-12T04:06:20Z → completed: 2026-05-12T04:08:00Z)
  Evidence: `./target/debug/crunch build bootstrap/gcc-4.0.ncl --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned --no-substitute` produced `.crunch-drain/store/qx1sjr104h3r6c55prik1nw63zbxl2j2-gcc-4.0.4`; artifact checks passed for `gcc`, `cc`, `cc1`, `crtbegin.o`, `crtend.o`, and `libgcc.a`; host `ar`/`nm` saw `_ashrdi3`, `_ashldi3`, `_lshrdi3`, `_muldi3`, and `_negdi2`; Nix clang-wrapper linked extracted `_ashrdi3.o` into a semantic smoke proving zero shift, positive `0x4000000000000000 >> 62 = 1`, negative sign extension for `-8 >> 1 = -4`, `-1 >> 63 = -1`, and `-0x4000000000000000 >> 60 = -4`.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate promote-gcc40-ashrdi3-libgcc-semantics --strict` and record evidence before archive.
