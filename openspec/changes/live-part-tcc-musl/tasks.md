## Implementation

- [ ] I1 Confirm `musl 1.1.24 and musl_target` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script.
- [ ] I2 Audit `bootstrap/tcc-musl.ncl` against the upstream part and record intentional Crunch deviations.
- [ ] I3 Fix `bootstrap/tcc-musl.ncl` so its source pins, patches, inputs, and output contract are self-contained.

## Verification

- [ ] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tcc-musl.ncl` and record the transcript.
- [ ] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/tcc-musl.ncl` with the documented bootstrap build environment and record output path plus elapsed time.
- [ ] V3 Smoke-test the produced output contract for `tcc linked to musl`.
- [ ] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage.
- [ ] V5 Run `openspec validate live-part-tcc-musl` after evidence is recorded.
