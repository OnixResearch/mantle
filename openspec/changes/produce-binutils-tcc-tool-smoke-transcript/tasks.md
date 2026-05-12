## Phase 1: Evidence Harness

- [x] [serial] Build/evaluate `bootstrap/binutils-tcc.ncl` into a closure-aware scratch store and identify the output path. ✅ 6m (started: 2026-05-12T20:05:00Z → completed: 2026-05-12T20:11:00Z; output: .crunch-drain/store-binutils-tcc-transcript/59vj0rbxkwj1wdn7kcak0i8zm132b1qn-binutils-2.30-tcc)
- [~] [depends:build-probe] Run required `as`, `ld`, `ar`, `ranlib`, `nm`, and `objcopy` smokes under bwrap with `/crunch/store` bound to the scratch store. ⏱ started: 2026-05-12T20:11:00Z
- [ ] [depends:tool-smokes] Write `bootstrap/evidence/binutils-tcc-tool-smoke.json` only if all required smokes pass, otherwise record the blocker under this change.
- [ ] [depends:transcript] Run parity report/tests and archive or leave the change with a concrete blocker.
