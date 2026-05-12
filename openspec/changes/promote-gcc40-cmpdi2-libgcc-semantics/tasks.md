## Phase 1: Implementation

- [x] [serial] Document `_cmpdi2` as the next semantic member in `bootstrap/gcc-4.0.ncl`. ✅ 1m (started: 2026-05-12T04:07:09Z → completed: 2026-05-12T04:08:00Z)
  Evidence: `bootstrap/gcc-4.0.ncl` comments now identify `_cmpdi2` as the sixth bounded semantic promotion using GCC libgcc's 0/1/2 signed less/equal/greater contract.
- [x] [serial] Replace the `_cmpdi2` placeholder body while preserving deterministic ar(5) archive generation. ✅ 1m (started: 2026-05-12T04:08:10Z → completed: 2026-05-12T04:09:20Z)
  Evidence: `_cmpdi2` now emits the GCC libgcc signed comparison contract (`0` less-than, `1` equal, `2` greater-than); `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` produced `gcc-4.0.4 script_bytes 57111 inputs 11`, `/bin/sh -n` on the generated builder passed, and `git diff --check` passed.
- [ ] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus `_cmpdi2` semantic smoke.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate promote-gcc40-cmpdi2-libgcc-semantics --strict` and record evidence before archive.
