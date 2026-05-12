## Phase 1: Implementation

- [ ] [serial] Document `_lshrdi3` as the next semantic member in `bootstrap/gcc-4.0.ncl`.
- [ ] [serial] Replace the `_lshrdi3` placeholder body while preserving deterministic ar(5) archive generation.
- [ ] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus `_lshrdi3` semantic smoke.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate promote-gcc40-lshrdi3-libgcc-semantics --strict` and record evidence before archive.
