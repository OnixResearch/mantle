## Implementation

- [x] I1 Confirm `heirloom devtools` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script.
- [x] I2 Audit `bootstrap/heirloom-devtools.ncl` against the upstream part and record intentional Crunch deviations.
- [x] I3 Fix `bootstrap/heirloom-devtools.ncl` so its source pins, patches, inputs, and output contract are self-contained.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/heirloom-devtools.ncl` and record the transcript.
- [x] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/heirloom-devtools.ncl` with the documented bootstrap build environment and record output path plus elapsed time.
- [x] V3 Smoke-test the produced output contract for `heirloom devtools`.
- [x] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage.
- [x] V5 Run `openspec validate live-part-heirloom-devtools` after evidence is recorded.
