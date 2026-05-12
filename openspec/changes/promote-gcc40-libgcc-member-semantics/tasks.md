## Phase 1: Implementation

- [ ] [serial] Select `_negdi2` or `_muldi3` and document the chosen semantics in the derivation comments.
- [ ] [serial] Replace the selected member body in `bootstrap/gcc-4.0.ncl` while preserving deterministic ar(5) archive generation.
- [ ] [serial] Rebuild `bootstrap/gcc-4.0.ncl` and verify artifact contract plus member semantic smoke.

## Phase 2: Completion

- [ ] [depends:implementation] Run `openspec validate promote-gcc40-libgcc-member-semantics --strict` and record evidence before archive.
