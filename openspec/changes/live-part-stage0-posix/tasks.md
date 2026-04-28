## Implementation

- [x] I1 Confirm `bootstrap-seeds through mescc-tools-extra` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. ✅ 3m (started: 2026-04-28T23:32:30Z → completed: 2026-04-28T23:35:40Z) [covers=bootstrap.part.stage0.posix] [evidence=evidence/I1-upstream-ordering.md]
- [x] I2 Audit `bootstrap/stage0-posix.ncl` against the upstream part and record intentional Crunch deviations. ✅ 4m (started: 2026-04-28T23:35:50Z → completed: 2026-04-28T23:36:05Z) [covers=bootstrap.part.stage0.posix] [evidence=evidence/I2-derivation-audit.md]
- [x] I3 Fix `bootstrap/stage0-posix.ncl` so its source pins, patches, inputs, and output contract are self-contained. ✅ 2m (started: 2026-04-28T23:36:05Z → completed: 2026-04-28T23:36:15Z) [covers=bootstrap.part.stage0.posix] [evidence=evidence/I3-self-contained-check.md]

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/stage0-posix.ncl` and record the transcript. ✅ 1m (started: 2026-04-28T23:36:20Z → completed: 2026-04-28T23:36:30Z) [covers=bootstrap.part.stage0.posix] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, source-pin audit reported `1 files, 7 fetch blocks, 0 issues`.
- [x] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/stage0-posix.ncl` with the documented bootstrap build environment and record output path plus elapsed time. ✅ 2m (started: 2026-04-28T23:36:50Z → completed: 2026-04-28T23:37:45Z) [covers=bootstrap.part.stage0.posix] [evidence=evidence/V2-build.md]
  - Evidence summary: PASS, rebuilt after the output-contract fix and produced `target/live-part-stage0-posix/store/35ljc87nc2gcn7cxpj078qjmch8qpqzh-stage0-posix` with `hermeticity: practical (no degraded facts)`.
- [x] V3 Smoke-test the produced output contract for `stage0-posix seed tools`. ✅ 3m (started: 2026-04-28T23:37:55Z → completed: 2026-04-28T23:39:50Z) [covers=bootstrap.part.stage0.posix] [evidence=evidence/V3-smoke.md]
  - Evidence summary: PASS, all declared tools are executable, `hex0` assembles one byte, and `get_machine` returns `amd64`.
- [ ] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage.
- [ ] V5 Run `openspec validate live-part-stage0-posix` after evidence is recorded.
