## Implementation

- [x] I1 Confirm `make 3.82` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. ✅ 1m (started: 2026-04-29T00:13:20Z → completed: 2026-04-29T00:13:42Z) [covers=bootstrap.part.make.3.82] [evidence=evidence/I1-upstream-ordering.md]
  - Evidence summary: PASS, this is the early tcc-built make; expected output is `bin/make` reporting GNU Make 3.82.
- [x] I2 Audit `bootstrap/make-tcc.ncl` against the upstream part and record intentional Crunch deviations. ✅ 1m (started: 2026-04-29T00:13:42Z → completed: 2026-04-29T00:13:47Z) [covers=bootstrap.part.make.3.82] [evidence=evidence/I2-derivation-audit.md]
  - Evidence summary: PARTIAL, source pin and source/object list match upstream pass1; build/smoke proof still required.
- [ ] I3 Fix `bootstrap/make-tcc.ncl` so its source pins, patches, inputs, and output contract are self-contained.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/make-tcc.ncl` and record the transcript. ✅ 1m (started: 2026-04-29T00:13:35Z → completed: 2026-04-29T00:13:38Z) [covers=bootstrap.part.make.3.82] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, `source-pin audit: 1 files, 1 fetch blocks, 0 issues`.
- [ ] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/make-tcc.ncl` with the documented bootstrap build environment and record output path plus elapsed time.
- [ ] V3 Smoke-test the produced output contract for `make 3.82`.
- [ ] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage.
- [ ] V5 Run `openspec validate live-part-make-3-82` after evidence is recorded.
