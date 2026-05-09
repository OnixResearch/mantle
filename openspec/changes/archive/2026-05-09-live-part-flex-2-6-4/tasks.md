## Implementation

- [x] I1 Confirm `flex 2.6.4` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script.
- [x] I2 Audit `bootstrap/flex-2.6.4-musl.ncl` against the upstream part and record intentional Crunch deviations.
- [x] I3 Fix `bootstrap/flex-2.6.4-musl.ncl` so its source pins, patches, inputs, and output contract are self-contained.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/flex-2.6.4-musl.ncl` and record the transcript.
- [x] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/flex-2.6.4-musl.ncl` with the documented bootstrap build environment and record output path plus elapsed time.
- [x] V3 Smoke-test the produced output contract for `flex 2.6.4`.
- [x] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage.
- [x] V5 Run `openspec validate live-part-flex-2-6-4` after evidence is recorded.
