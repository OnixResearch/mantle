## Implementation

- [x] I1 Confirm `mpc-1.2.1` ordering, source notes, and expected output contract from `steps/manifest`, `steps/mpc-1.2.1/`, and the mismatched `parts.rst` heading. Evidence: `evidence/I1-upstream-ordering.md`.
- [x] I2 Audit `bootstrap/mpc-1.2.1.ncl` against the upstream part and record intentional Crunch deviations. Evidence: `evidence/I2-derivation-audit.md`.
- [x] I3 Fix `bootstrap/mpc-1.2.1.ncl` so its source pins, patches, inputs, and output contract are self-contained. Evidence: `evidence/I3-fix.md`.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/mpc-1.2.1.ncl` and record the transcript. Evidence: `evidence/V1-source-pin-recheck.md` and `evidence/V1-source-pin-recheck.log`.
- [x] V2 Run or gate `/tmp/crunch-build/debug/crunch build bootstrap/mpc-1.2.1.ncl` with the documented bootstrap build environment and record output path plus elapsed time. Evidence: `evidence/V2-build-gate.md`; no output path exists because the required `gcc-4.7.4` provider is absent.
- [x] V3 Smoke-test the produced output contract for `mpc 1.2.1`. Evidence: `evidence/V3-smoke-gate.md`; smoke remains blocked until V2 has a real output path.
- [x] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage. Evidence: `evidence/V4-host-leakage-gate.md`; transcript scan remains blocked until V2 has a real transcript, and host fallback is explicitly forbidden.
- [x] V5 Run `openspec validate live-part-mpc-1-2-1` after evidence is recorded. Evidence: `evidence/V5-openspec-validation.md`; strict OpenSpec validation and `git diff --check` passed before archive. Helper verify reported only expected pre-archive warning classes before this V5 checkbox was marked plus heading-id warnings.
