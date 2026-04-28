## Implementation

- [x] I1 Confirm `bootstrap-seeds through mescc-tools-extra` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. ✅ 3m (started: 2026-04-28T23:32:30Z → completed: 2026-04-28T23:35:40Z) [covers=bootstrap.part.stage0.posix] [evidence=evidence/I1-upstream-ordering.md]
- [ ] I2 Audit `bootstrap/stage0-posix.ncl` against the upstream part and record intentional Crunch deviations.
- [ ] I3 Fix `bootstrap/stage0-posix.ncl` so its source pins, patches, inputs, and output contract are self-contained.

## Verification

- [ ] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/stage0-posix.ncl` and record the transcript.
- [ ] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/stage0-posix.ncl` with the documented bootstrap build environment and record output path plus elapsed time.
- [ ] V3 Smoke-test the produced output contract for `stage0-posix seed tools`.
- [ ] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage.
- [ ] V5 Run `openspec validate live-part-stage0-posix` after evidence is recorded.
