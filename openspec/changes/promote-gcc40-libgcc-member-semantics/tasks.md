## Phase 1: Implementation

- [x] [serial] Select `_negdi2` or `_muldi3` and document the chosen semantics in the derivation comments. ✅ 1m (started: 2026-05-12T03:22:55Z → completed: 2026-05-12T03:23:29Z)
  Evidence: selected `_negdi2` as the first semantic member and documented two's-complement signed 64-bit negation in `bootstrap/gcc-4.0.ncl`.
- [ ] [serial] Replace the selected member body in `bootstrap/gcc-4.0.ncl` while preserving deterministic ar(5) archive generation.
- [ ] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus member semantic smoke.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate promote-gcc40-libgcc-member-semantics --strict` and record evidence before archive.
