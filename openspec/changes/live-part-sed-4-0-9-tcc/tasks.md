## Implementation

- [x] I1 Confirm `sed 4.0.9` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. Evidence: `evidence/I1-upstream-ordering.md`.
- [x] I2 Audit `bootstrap/sed-tcc.ncl` against the upstream part and record intentional Crunch deviations. Evidence: `evidence/I2-derivation-audit.md`.
- [x] I3 Fix `bootstrap/sed-tcc.ncl` so its source pins, patches, inputs, and output contract are self-contained. Evidence: `evidence/I3-fix.md`.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/sed-tcc.ncl` and record the transcript. Evidence: `evidence/I3-source-pin-recheck.log`.
- [ ] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/sed-tcc.ncl` with the documented bootstrap build environment and record output path plus elapsed time.
- [ ] V3 Smoke-test the produced output contract for `sed 4.0.9 (tcc)`.
- [ ] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage.
- [ ] V5 Run `openspec validate live-part-sed-4-0-9-tcc` after evidence is recorded.
