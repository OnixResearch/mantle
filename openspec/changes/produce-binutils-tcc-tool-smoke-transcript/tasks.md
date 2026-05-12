## Phase 1: Evidence Harness

- [ ] [serial] Build/evaluate `bootstrap/binutils-tcc.ncl` into a closure-aware scratch store and identify the output path.
- [ ] [depends:build-probe] Run required `as`, `ld`, `ar`, `ranlib`, `nm`, and `objcopy` smokes under bwrap with `/crunch/store` bound to the scratch store.
- [ ] [depends:tool-smokes] Write `bootstrap/evidence/binutils-tcc-tool-smoke.json` only if all required smokes pass, otherwise record the blocker under this change.
- [ ] [depends:transcript] Run parity report/tests and archive or leave the change with a concrete blocker.
