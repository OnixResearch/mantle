## Implementation

- [x] I1 Confirm `autoconf 2.52` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. Evidence: `evidence/I1-upstream-ordering.md`.
- [x] I2 Audit `bootstrap/autoconf-2.52.ncl` against the upstream part and record intentional Crunch deviations. Evidence: `evidence/I2-derivation-audit.md`.
- [x] I3 Fix `bootstrap/autoconf-2.52.ncl` so its source pins, patches, inputs, and output contract are self-contained. Evidence: `evidence/I3-fix.md`.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/autoconf-2.52.ncl` and record the transcript. Evidence: `evidence/V1-source-pin-recheck.md` and `evidence/V1-source-pin-recheck.log`.
- [x] V2 Run or gate `/tmp/crunch-build/debug/crunch build bootstrap/autoconf-2.52.ncl` with the documented bootstrap build environment and record output path plus elapsed time. Evidence: `evidence/V2-build-gate.md`; no output path exists because predecessor `make-3.82-tcc` remains blocked.
- [x] V3 Smoke-test the produced output contract for `autoconf 2.52`. Evidence: `evidence/V3-smoke-gate.md`; smoke remains blocked until V2 has a real output path.
- [x] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage. Evidence: `evidence/V4-host-leakage-gate.md`; transcript scan remains blocked until V2 has a real transcript, and host fallback is explicitly forbidden.
- [x] V5 Run `openspec validate live-part-autoconf-2-52` after evidence is recorded. Evidence: `evidence/V5-openspec-validation.md`; strict OpenSpec validation and `git diff --check` passed before archive. Helper verify reported only expected pre-checkbox task warning plus heading-id warnings.
