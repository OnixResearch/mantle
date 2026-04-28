## Implementation

- [ ] I1 Confirm `patch 2.5.9` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script.
- [ ] I2 Audit `bootstrap/patch-tcc.ncl` against the upstream part and record intentional Crunch deviations.
- [ ] I3 Fix `bootstrap/patch-tcc.ncl` so its source pins, patches, inputs, and output contract are self-contained.

## Verification

- [ ] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/patch-tcc.ncl` and record the transcript.
- [ ] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/patch-tcc.ncl` with the documented bootstrap build environment and record output path plus elapsed time.
- [ ] V3 Smoke-test the produced output contract for `patch 2.5.9`.
- [ ] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage.
- [ ] V5 Run `openspec validate live-part-patch-2-5-9` after evidence is recorded.
