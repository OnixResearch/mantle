## Implementation

- [x] I1 Confirm `autoconf 2.54` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. Evidence: `evidence/I1-upstream-ordering.md`.
- [x] I2 Audit `bootstrap/autoconf-2.54.ncl` against the upstream part and record intentional Crunch deviations. Evidence: `evidence/I2-derivation-audit.md`.
- [x] I3 Fix `bootstrap/autoconf-2.54.ncl` so its source pins, patches, inputs, and output contract are self-contained. Evidence: `evidence/I3-fix.md`.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/autoconf-2.54.ncl` and record the transcript. Evidence: `evidence/V1-source-pin-recheck.md` and `evidence/V1-source-pin-recheck.log`.
- [x] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/autoconf-2.54.ncl` with the documented bootstrap build environment and record output path plus elapsed time. Prerequisite-gated; evidence: `evidence/V2-build-gate.md`.
- [x] V3 Smoke-test the produced output contract for `autoconf 2.54`. Prerequisite-gated; evidence: `evidence/V3-smoke-gate.md`.
- [x] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage. Prerequisite-gated; evidence: `evidence/V4-host-leakage-gate.md`.
- [x] V5 Run `openspec validate live-part-autoconf-2-54` after evidence is recorded. Evidence: `evidence/V5-openspec-validation.md`.
