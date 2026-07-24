## Phase 1: Runtime boundary

- [x] [serial] I1 Record the current bridge baseline and mechanism-level approach registry, including replacement, cache-preseed, and runtime-confinement routes. r[gcc40_bridge.configure_preprocess_confinement]
  - Evidence: `evidence/baseline.md` records three distinct mechanisms, false-completion cases, the advisory-audit rejection, and the bounded runtime-confinement route.
- [x] [serial] I2 Enforce canonical configure directory, source, output, byte, class, and invocation-count bounds before compiler execution. r[gcc40_bridge.configure_preprocess_confinement]
  - Evidence: `bootstrap/gcc-4.0-native.ncl` now rejects missing/mismatched authority, unknown classes, non-`conftest.c`, canonical-parent escape, explicit output, probes over 65,536 bytes, and invocation count exhaustion before its first compiler command.
- [x] [serial] I3 Emit and verify a bounded build-local configure bridge audit without broadening provider claims. r[gcc40_bridge.configure_preprocess_confinement]
  - Evidence: each admitted request appends count/class/authority/source before compiler execution; the parent requires the audit, bounds its count, and validates every class, directory, and source after configure.

## Phase 2: Deterministic evidence

- [x] [serial] I4 Add the checked bridge contract and a functional-core checker bound to exact runtime markers. r[gcc40_bridge.configure_preprocess_confinement]
  - Evidence: `bootstrap/evidence/gcc-4.0-configure-preprocess-bridge.json` and `scripts/check-gcc40-configure-bridge.rs` bind limits, classes, source/output policy, audit order, provider ineligibility, and non-claims.
- [x] [serial] V1 Add positive and negative tests for every admission fact, policy/evidence drift, and compiler-before-rejection false completion. r[gcc40_bridge.configure_preprocess_confinement]
  - Evidence: checker self-tests admit the exact boundary and reject ten runtime fact failures plus source-marker and provider-eligibility drift; the source-order check requires durable audit before the first compiler marker.
- [x] [serial] V2 Run focused bootstrap, Tiger Style, first-party quality, blocker, machine-contract, Cairn, Tracey, and full Nix gates; preserve exact broad-gate blockers. r[gcc40_bridge.configure_preprocess_confinement]
  - Evidence: `evidence/validation.md` records the 19-test bootstrap rail, bounded checker negatives, 437 evidence-backed blocker suppressions with zero findings, strict machine-contract freshness, 145/145 Tracey coverage, first-party Rustfmt/Clippy/serialized tests, and the full build-mode Nix result (`4048 passed`, `all checks passed!`).
- [ ] [serial] V3 Sync, inspect, archive, and commit the accepted bounded non-claim with exact evidence. r[gcc40_bridge.configure_preprocess_confinement]