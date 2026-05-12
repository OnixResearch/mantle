## Phase 1: Implementation

- [x] [serial] Document `_cmpdi2` as the next semantic member in `bootstrap/gcc-4.0.ncl`. ✅ 1m (started: 2026-05-12T04:07:09Z → completed: 2026-05-12T04:08:00Z)
  Evidence: `bootstrap/gcc-4.0.ncl` comments now identify `_cmpdi2` as the sixth bounded semantic promotion using GCC libgcc's 0/1/2 signed less/equal/greater contract.
- [x] [serial] Replace the `_cmpdi2` placeholder body while preserving deterministic ar(5) archive generation. ✅ 1m (started: 2026-05-12T04:08:10Z → completed: 2026-05-12T04:09:20Z)
  Evidence: `_cmpdi2` now emits the GCC libgcc signed comparison contract (`0` less-than, `1` equal, `2` greater-than); `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` produced `gcc-4.0.4 script_bytes 57111 inputs 11`, `/bin/sh -n` on the generated builder passed, and `git diff --check` passed.
- [x] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus `_cmpdi2` semantic smoke. ✅ 2m (started: 2026-05-12T04:09:35Z → completed: 2026-05-12T04:11:10Z)
  Evidence: `./target/debug/crunch build bootstrap/gcc-4.0.ncl --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned --no-substitute` produced `.crunch-drain/store/d0i62idvv14k3hfff6104v5vh94v5jq7-gcc-4.0.4`; artifact checks passed for `gcc`, `cc`, `cc1`, `crtbegin.o`, `crtend.o`, and `libgcc.a`; host `ar`/`nm` saw `_cmpdi2`, `_ashrdi3`, `_ashldi3`, `_lshrdi3`, `_muldi3`, and `_negdi2`; Nix clang-wrapper linked extracted `_cmpdi2.o` into a semantic smoke proving signed less/equal/greater outputs for negative, zero, positive, max, and min 64-bit values.

## Phase 2: Completion

- [x] [depends:implementation] Run `openspec validate promote-gcc40-cmpdi2-libgcc-semantics --strict` and record evidence before archive. ✅ 1m (started: 2026-05-12T04:11:20Z → completed: 2026-05-12T04:11:35Z)
  Evidence: `openspec validate promote-gcc40-cmpdi2-libgcc-semantics --strict` passed before archive.
