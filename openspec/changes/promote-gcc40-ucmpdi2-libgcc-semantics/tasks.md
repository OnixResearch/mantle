## Phase 1: Implementation

- [ ] [serial] Document `_ucmpdi2` as the next semantic member in `bootstrap/gcc-4.0.ncl`.
- [ ] [serial] Replace the `_ucmpdi2` placeholder body while preserving deterministic ar(5) archive generation.
- [ ] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus `_ucmpdi2` semantic smoke.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate promote-gcc40-ucmpdi2-libgcc-semantics --strict` and record evidence before archive.
