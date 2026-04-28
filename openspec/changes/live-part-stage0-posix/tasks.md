## Implementation

- [x] I1 Confirm `bootstrap-seeds through mescc-tools-extra` ordering, source notes, and expected output contract from `parts.rst` plus upstream seed scripts `seed/seed.kaem`, `seed/after.kaem`, and `seed/preseeded.kaem` (this boundary has no `steps/` directory). ✅ 3m (started: 2026-04-28T23:32:30Z → completed: 2026-04-28T23:35:40Z) [covers=bootstrap.part.stage0.posix] [evidence=evidence/I1-upstream-ordering.md]
- [x] I2 Audit `bootstrap/stage0-posix.ncl` against the upstream part and record intentional Crunch deviations. ✅ 4m (started: 2026-04-28T23:35:50Z → completed: 2026-04-28T23:36:05Z) [covers=bootstrap.part.stage0.posix] [evidence=evidence/I2-derivation-audit.md]
- [x] I3 Fix `bootstrap/stage0-posix.ncl` so its source pins, patches, inputs, and output contract are self-contained. ✅ 2m (started: 2026-04-28T23:36:05Z → completed: 2026-04-28T23:36:15Z) [covers=bootstrap.part.stage0.posix] [evidence=evidence/I3-self-contained-check.md]
- [x] I4 Update the umbrella roll-up dependency handoff so broad live-bootstrap changes consume this part's evidence instead of duplicating detailed state. ✅ 2m (started: 2026-04-28T23:41:30Z → completed: 2026-04-28T23:41:50Z) [covers=bootstrap.part.stage0.posix] [evidence=evidence/I4-umbrella-handoff.md]
  - Evidence summary: PASS, `live-bootstrap-parts-index.md` records the completed `live-part-stage0-posix` handoff and `live-bootstrap-source-chain` V1 names this part as the stage0 evidence owner.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/stage0-posix.ncl` and record the transcript. ✅ 1m (started: 2026-04-28T23:36:20Z → completed: 2026-04-28T23:36:30Z) [covers=bootstrap.part.stage0.posix] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, source-pin audit reported `1 files, 7 fetch blocks, 0 issues`.
- [x] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/stage0-posix.ncl` with the documented bootstrap build environment and record output path plus elapsed time. ✅ 2m (started: 2026-04-28T23:36:50Z → completed: 2026-04-28T23:37:45Z) [covers=bootstrap.part.stage0.posix] [evidence=evidence/V2-build.md]
  - Evidence summary: PASS, rebuilt after the output-contract fix and produced `target/live-part-stage0-posix/store/35ljc87nc2gcn7cxpj078qjmch8qpqzh-stage0-posix` with `hermeticity: practical (no degraded facts)`.
- [x] V3 Smoke-test the produced output contract for `stage0-posix seed tools`. ✅ 3m (started: 2026-04-28T23:37:55Z → completed: 2026-04-28T23:39:50Z) [covers=bootstrap.part.stage0.posix] [evidence=evidence/V3-smoke.md]
  - Evidence summary: PASS, all declared tools are executable, `hex0` assembles one byte, and `get_machine` returns `amd64`.
- [x] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage. ✅ 1m (started: 2026-04-28T23:40:00Z → completed: 2026-04-28T23:40:20Z) [covers=bootstrap.part.stage0.posix] [evidence=evidence/V4-host-leakage.md]
  - Evidence summary: PASS, derivation and stage-local build transcript have no forbidden `/usr`, Nix command, musl.cc, host home, Cargo, clang, or gcc references; only declared sandbox `/bin/sh` and `/bin/busybox` are present.
- [x] V5 Run `openspec validate live-part-stage0-posix` after evidence is recorded. ✅ 1m (started: 2026-04-28T23:40:35Z → completed: 2026-04-28T23:40:45Z) [covers=bootstrap.part.stage0.posix] [evidence=evidence/V5-openspec-validate.md]
  - Evidence summary: PASS, `openspec validate live-part-stage0-posix` reported `Change 'live-part-stage0-posix' is valid`.
