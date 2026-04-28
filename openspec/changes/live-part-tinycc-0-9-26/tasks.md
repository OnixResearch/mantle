## Evidence format

Every task records evidence under `openspec/changes/live-part-tinycc-0-9-26/evidence/`. Markdown evidence files include `Task-ID:` and `Covers:` metadata. Raw command transcripts and scans are stored beside the markdown evidence as `*-full.log`, `*-output.txt`, or `*-transcript-only.log` files and are referenced from the owning task.

## Implementation

- [x] I1 Confirm `tinycc 0.9.26` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. ✅ 3m (started: 2026-04-28T23:48:10Z → completed: 2026-04-28T23:49:13Z) [covers=bootstrap.part.tinycc.0.9.26] [evidence=evidence/I1-upstream-ordering.md]
- [x] I2 Audit `bootstrap/tinycc-mes.ncl` against the upstream part and record intentional Crunch deviations. ✅ 2m (started: 2026-04-28T23:49:13Z → completed: 2026-04-28T23:49:19Z) [covers=bootstrap.part.tinycc.0.9.26] [evidence=evidence/I2-derivation-audit.md]
  - Evidence summary: PARTIAL/BLOCKED for later build proof; derivation mirrors upstream sequence, but V2/V3 must still prove real self-compile beyond `tcc-mes -version`.
- [ ] I3 Fix `bootstrap/tinycc-mes.ncl` so its source pins, patches, inputs, and output contract are self-contained.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tinycc-mes.ncl` and record the transcript. ✅ 1m (started: 2026-04-28T23:48:20Z → completed: 2026-04-28T23:49:01Z) [covers=bootstrap.part.tinycc.0.9.26] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, source-pin audit reported `1 files, 2 fetch blocks, 0 issues`.
- [ ] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/tinycc-mes.ncl` with the documented bootstrap build environment and record output path plus elapsed time.
- [ ] V3 Smoke-test the produced output contract for `tinycc 0.9.26`.
- [ ] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage.
- [ ] V5 Run `openspec validate live-part-tinycc-0-9-26` after evidence is recorded.
