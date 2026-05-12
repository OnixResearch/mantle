## Phase 1: Implementation

- [x] [serial] Document `_ucmpdi2` as the next semantic member in `bootstrap/gcc-4.0.ncl`. ✅ 1m (started: 2026-05-12T04:11:45Z → completed: 2026-05-12T04:12:20Z)
  Evidence: `bootstrap/gcc-4.0.ncl` comments now identify `_ucmpdi2` as the seventh bounded semantic promotion using GCC libgcc's 0/1/2 unsigned less/equal/greater contract.
- [x] [serial] Replace the `_ucmpdi2` placeholder body while preserving deterministic ar(5) archive generation. ✅ 1m (started: 2026-05-12T04:12:25Z → completed: 2026-05-12T04:13:25Z)
  Evidence: `_ucmpdi2` now emits the GCC libgcc unsigned comparison contract (`0` less-than, `1` equal, `2` greater-than); `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` produced `gcc-4.0.4 script_bytes 57437 inputs 11`, `/bin/sh -n` on the generated builder passed, and `git diff --check` passed.
- [x] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus `_ucmpdi2` semantic smoke. ✅ 2m (started: 2026-05-12T04:13:35Z → completed: 2026-05-12T04:15:10Z)
  Evidence: `./target/debug/crunch build bootstrap/gcc-4.0.ncl --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned --no-substitute` produced `.crunch-drain/store/d3dzlm6s68i9mcd5v38zma6ajxrfccsb-gcc-4.0.4`; artifact checks passed for `gcc`, `cc`, `cc1`, `crtbegin.o`, `crtend.o`, and `libgcc.a`; host `ar`/`nm` saw `_ucmpdi2`, `_cmpdi2`, `_ashrdi3`, `_ashldi3`, `_lshrdi3`, `_muldi3`, and `_negdi2`; Nix clang-wrapper linked extracted `_ucmpdi2.o` into a semantic smoke proving unsigned less/equal/greater outputs across low values, high-bit ordering, and all-ones equality.

## Phase 2: Completion

- [x] [depends:implementation] Run `openspec validate promote-gcc40-ucmpdi2-libgcc-semantics --strict` and record evidence before archive. ✅ 1m (started: 2026-05-12T04:15:20Z → completed: 2026-05-12T04:15:35Z)
  Evidence: `openspec validate promote-gcc40-ucmpdi2-libgcc-semantics --strict` passed before archive.
