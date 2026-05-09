## Implementation

- [x] I1 Confirm `mpfr 4.1.0` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script.
- [x] I2 Audit `bootstrap/mpfr-4.1.0.ncl` against the upstream part and record intentional Crunch deviations.
- [x] I3 Fix `bootstrap/mpfr-4.1.0.ncl` so its source pins, patches, inputs, and output contract are self-contained.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/mpfr-4.1.0.ncl` and record the transcript.
- [x] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/mpfr-4.1.0.ncl` with the documented bootstrap build environment and record output path plus elapsed time.
- [x] V3 Smoke-test the produced output contract for `mpfr 4.1.0`.
- [x] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage.
- [x] V5 Run `openspec validate live-part-mpfr-4-1-0` after evidence is recorded.
