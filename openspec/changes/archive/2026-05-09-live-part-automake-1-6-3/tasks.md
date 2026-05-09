## Implementation

- [x] I1 Confirm `automake 1.6.3` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script.
- [x] I2 Audit `bootstrap/automake-1.6.3.ncl` against the upstream part and record intentional Crunch deviations.
- [x] I3 Fix `bootstrap/automake-1.6.3.ncl` so its source pins, patches, inputs, and output contract are self-contained.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/automake-1.6.3.ncl` and record the transcript.
- [x] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/automake-1.6.3.ncl` with the documented bootstrap build environment and record output path plus elapsed time.
- [x] V3 Smoke-test the produced output contract for `automake 1.6.3`.
- [x] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage.
- [x] V5 Run `openspec validate live-part-automake-1-6-3` after evidence is recorded.
