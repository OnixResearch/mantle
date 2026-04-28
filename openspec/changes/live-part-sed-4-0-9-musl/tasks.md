## Implementation

- [ ] I1 Confirm `sed 4.0.9` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script.
- [ ] I2 Audit `bootstrap/sed-4.0.9-musl.ncl` against the upstream part and record intentional Crunch deviations.
- [ ] I3 Fix `bootstrap/sed-4.0.9-musl.ncl` so its source pins, patches, inputs, and output contract are self-contained.

## Verification

- [ ] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/sed-4.0.9-musl.ncl` and record the transcript.
- [ ] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/sed-4.0.9-musl.ncl` with the documented bootstrap build environment and record output path plus elapsed time.
- [ ] V3 Smoke-test the produced output contract for `sed 4.0.9 (musl)`.
- [ ] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage.
- [ ] V5 Run `openspec validate live-part-sed-4-0-9-musl` after evidence is recorded.
