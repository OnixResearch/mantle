## Phase 1: Implementation

- [x] [serial] Select `_negdi2` or `_muldi3` and document the chosen semantics in the derivation comments. ✅ 1m (started: 2026-05-12T03:22:55Z → completed: 2026-05-12T03:23:29Z)
  Evidence: selected `_negdi2` as the first semantic member and documented two's-complement signed 64-bit negation in `bootstrap/gcc-4.0.ncl`.
- [x] [serial] Replace the selected member body in `bootstrap/gcc-4.0.ncl` while preserving deterministic ar(5) archive generation. ✅ 1m (started: 2026-05-12T03:23:49Z → completed: 2026-05-12T03:24:38Z)
  Evidence: `_negdi2` now emits `long long _negdi2(long long value) { return -value; }`; `crunch eval bootstrap/gcc-4.0.ncl` and `/bin/sh -n` on the generated builder passed.
- [x] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus member semantic smoke. ✅ 1m (started: 2026-05-12T03:25:03Z → completed: 2026-05-12T03:26:06Z)
  Evidence: `./target/debug/crunch build bootstrap/gcc-4.0.ncl --store "$PWD/.crunch-drain/store" -j 4 --trust-unsigned --no-substitute` produced `.crunch-drain/store/m7b8dp6vmf9v8xqqv1x5q1biymdsq70y-gcc-4.0.4`; artifact contract passed for `gcc`, `cc`, `cc1`, `libgcc.a`, `crtbegin.o`, and `crtend.o`; host `ar`/`nm` saw `_negdi2` and `__gcc_bcmp`; linked semantic smoke proved `_negdi2(1)=-1`, `_negdi2(-7)=7`, `_negdi2(0)=0`, and `_negdi2(1234567890123)=-1234567890123`.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate promote-gcc40-libgcc-member-semantics --strict` and record evidence before archive.
