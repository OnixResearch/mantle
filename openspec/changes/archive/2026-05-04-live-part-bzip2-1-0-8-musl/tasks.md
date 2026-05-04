## Implementation

- [x] I1 Confirm `bzip2 1.0.8` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. Evidence: `evidence/I1-upstream-ordering.md`.
- [x] I2 Audit `bootstrap/bzip2-1.0.8-musl.ncl` against the upstream part and record intentional Crunch deviations. Evidence: `evidence/I2-derivation-audit.md`.
- [x] I3 Fix `bootstrap/bzip2-1.0.8-musl.ncl` so its source pins, patches, inputs, and output contract are self-contained. Evidence: `evidence/I3-fix.md`.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/bzip2-1.0.8-musl.ncl` and record the transcript. Evidence: `evidence/V1-source-pin-audit.log`.
- [x] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/bzip2-1.0.8-musl.ncl` with the documented bootstrap build environment and record output path plus elapsed time. Evidence: `evidence/V2-build.md`.
- [x] V3 Smoke-test the produced output contract for `bzip2 1.0.8 (musl)`. Evidence: `evidence/V3-smoke.md`.
- [x] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage. Evidence: `evidence/V4-host-leakage.md`.
- [x] V5 Run `openspec validate live-part-bzip2-1-0-8-musl` after evidence is recorded. Evidence: `evidence/V5-openspec-validate.md`.
