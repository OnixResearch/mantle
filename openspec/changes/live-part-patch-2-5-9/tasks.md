## Implementation

- [x] I1 Confirm `patch 2.5.9` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. ✅ 1m (started: 2026-04-29T00:15:13Z → completed: 2026-04-29T00:15:31Z) [covers=bootstrap.part.patch.2.5.9] [evidence=evidence/I1-upstream-ordering.md]
  - Evidence summary: PASS, patch follows make and supplies complex line-edit support; expected output is `bin/patch`.
- [x] I2 Audit `bootstrap/patch-tcc.ncl` against the upstream part and record intentional Crunch deviations. ✅ 1m (started: 2026-04-29T00:15:31Z → completed: 2026-04-29T00:15:35Z) [covers=bootstrap.part.patch.2.5.9] [evidence=evidence/I2-derivation-audit.md]
  - Evidence summary: PARTIAL, fixed source and output assertions exist; recipe manually lists CFLAGS/objects instead of upstream `mk/main.mk`, so V2/V3 proof remains required.
- [x] I3 Fix `bootstrap/patch-tcc.ncl` so its source pins, patches, inputs, and output contract are self-contained. ✅ 5m (started: 2026-04-30T22:33:00Z → completed: 2026-04-30T22:38:00Z) [covers=bootstrap.part.patch.2.5.9] [evidence=evidence/I3-fix.md]

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/patch-tcc.ncl` and record the transcript. ✅ 1m (started: 2026-04-29T00:15:24Z → completed: 2026-04-29T00:15:28Z) [covers=bootstrap.part.patch.2.5.9] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, `source-pin audit: 1 files, 1 fetch blocks, 0 issues`.
- [x] V2 Deferred to openspec change: `live-part-patch-2-5-9-runtime-validation` ✅ 0m (deferred; blocked by prerequisite make/tcc runtime validation) [covers=bootstrap.part.patch.2.5.9] [evidence=evidence/V2-build.md]
- [x] V3 Deferred to openspec change: `live-part-patch-2-5-9-runtime-validation` ✅ 0m (deferred; depends on V2 output) [covers=bootstrap.part.patch.2.5.9] [evidence=evidence/V3-smoke.md]
- [x] V4 Deferred to openspec change: `live-part-patch-2-5-9-runtime-validation` ✅ 0m (deferred; depends on V2 transcript) [covers=bootstrap.part.patch.2.5.9] [evidence=evidence/V4-host-leakage.md]
- [x] V5 Run `openspec validate live-part-patch-2-5-9` after evidence is recorded. ✅ 1m (warnings only: delta-spec heading IDs) [covers=bootstrap.part.patch.2.5.9] [evidence=evidence/V5-openspec-validate.md]
