## Phase 1: Implementation

- [x] [serial] Document `_ucmpdi2` as the next semantic member in `bootstrap/gcc-4.0.ncl`. ✅ 1m (started: 2026-05-12T04:11:45Z → completed: 2026-05-12T04:12:20Z)
  Evidence: `bootstrap/gcc-4.0.ncl` comments now identify `_ucmpdi2` as the seventh bounded semantic promotion using GCC libgcc's 0/1/2 unsigned less/equal/greater contract.
- [x] [serial] Replace the `_ucmpdi2` placeholder body while preserving deterministic ar(5) archive generation. ✅ 1m (started: 2026-05-12T04:12:25Z → completed: 2026-05-12T04:13:25Z)
  Evidence: `_ucmpdi2` now emits the GCC libgcc unsigned comparison contract (`0` less-than, `1` equal, `2` greater-than); `./target/debug/crunch eval bootstrap/gcc-4.0.ncl` produced `gcc-4.0.4 script_bytes 57437 inputs 11`, `/bin/sh -n` on the generated builder passed, and `git diff --check` passed.
- [ ] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus `_ucmpdi2` semantic smoke.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate promote-gcc40-ucmpdi2-libgcc-semantics --strict` and record evidence before archive.
