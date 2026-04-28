## Evidence format

Every task records evidence under `openspec/changes/live-part-tinycc-0-9-27/evidence/`. Markdown evidence files include `Task-ID:` and `Covers:` metadata. Raw command transcripts and scans are stored beside the markdown evidence as `*-full.log`, `*-output.txt`, or `*-transcript-only.log` files and are referenced from the owning task.

## Implementation

- [x] I1 Confirm `tinycc 0.9.27` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. ✅ 2m (started: 2026-04-28T23:50:03Z → completed: 2026-04-28T23:50:17Z) [covers=bootstrap.part.tinycc.0.9.27] [evidence=evidence/I1-upstream-ordering.md]
- [x] I2 Audit `bootstrap/tinycc.ncl` against the upstream part and record intentional Crunch deviations. ✅ 2m (started: 2026-04-28T23:50:17Z → completed: 2026-04-28T23:50:23Z) [covers=bootstrap.part.tinycc.0.9.27] [evidence=evidence/I2-derivation-audit.md]
  - Evidence summary: PARTIAL, dependency/source order matches but missing upstream patch/runtime-rebuild steps are carried into I3/V2.
- [ ] I3 Fix `bootstrap/tinycc.ncl` so its source pins, patches, inputs, and output contract are self-contained.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tinycc.ncl` and record the transcript. ✅ 1m (started: 2026-04-28T23:50:10Z → completed: 2026-04-28T23:50:12Z) [covers=bootstrap.part.tinycc.0.9.27] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, source-pin audit reported `1 files, 1 fetch blocks, 0 issues`.
- [ ] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/tinycc.ncl` with the documented bootstrap build environment and record output path plus elapsed time.
- [ ] V3 Smoke-test the produced output contract for `tinycc 0.9.27`.
- [ ] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage.
- [ ] V5 Run `openspec validate live-part-tinycc-0-9-27` after evidence is recorded.
