## Phase 1: Implementation

- [ ] [serial] Document `_ashrdi3` as the next semantic member in `bootstrap/gcc-4.0.ncl`.
- [ ] [serial] Replace the `_ashrdi3` placeholder body while preserving deterministic ar(5) archive generation.
- [ ] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus `_ashrdi3` semantic smoke.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate promote-gcc40-ashrdi3-libgcc-semantics --strict` and record evidence before archive.
