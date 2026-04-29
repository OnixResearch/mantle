## Evidence format

Every task records evidence under `openspec/changes/live-part-tinycc-0-9-26/evidence/`. Markdown evidence files include `Task-ID:` and `Covers:` metadata. Raw command transcripts and scans are stored beside the markdown evidence as `*-full.log`, `*-output.txt`, or `*-transcript-only.log` files and are referenced from the owning task.

## Implementation

- [x] I1 Confirm `tinycc 0.9.26` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. ✅ 3m (started: 2026-04-28T23:48:10Z → completed: 2026-04-28T23:49:13Z) [covers=bootstrap.part.tinycc.0.9.26] [evidence=evidence/I1-upstream-ordering.md]
- [x] I2 Audit `bootstrap/tinycc-mes.ncl` against the upstream part and record intentional Crunch deviations. ✅ 2m (started: 2026-04-28T23:49:13Z → completed: 2026-04-28T23:49:19Z) [covers=bootstrap.part.tinycc.0.9.26] [evidence=evidence/I2-derivation-audit.md]
  - Evidence summary: PARTIAL/BLOCKED for later build proof; derivation mirrors upstream sequence, but V2/V3 must still prove real self-compile beyond `tcc-mes -version`.
- [x] I3 Fix `bootstrap/tinycc-mes.ncl` so its source pins, patches, inputs, and output contract are self-contained. ✅ 1m (started: 2026-04-29T13:31:00Z → completed: 2026-04-29T13:32:00Z) [covers=bootstrap.part.tinycc.0.9.26] [evidence=evidence/I3-fix.md]
  - Evidence summary: PASS, archived blocker fix proves `tcc-mes` compiles `tcc-boot0` without segfaulting and the shipped compiler/output contract stays self-contained.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tinycc-mes.ncl` and record the transcript. ✅ 1m (started: 2026-04-28T23:48:20Z → completed: 2026-04-28T23:49:01Z) [covers=bootstrap.part.tinycc.0.9.26] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, source-pin audit reported `1 files, 2 fetch blocks, 0 issues`.
- [x] V2 Run `CRUNCH_NO_FUSE=1 ./target/debug/crunch build bootstrap/tinycc-mes.ncl --no-substitute -j 1 --verbose --log-level info` with the documented bootstrap build environment and record output path plus elapsed time. ✅ 8m (completed: 2026-04-29T14:33:44Z) [covers=bootstrap.part.tinycc.0.9.26] [evidence=evidence/V2-build-success.md]
  - Evidence summary: PASS, archived blocker-fix transcript records successful output `/home/brittonr/git/crunch/crunch/target/fix-tinycc-mes-bufferedfile-codegen/run-current-nofuse/store/miymhdmqink0c81drn4y1f1chc73vdf9-tinycc-0.9.26`, `tcc-boot0` compilation, no `Segmentation fault`, and `hermeticity: practical (no degraded facts)`.
- [ ] V3 Smoke-test the produced output contract for `tinycc 0.9.26`.
- [ ] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage.
- [ ] V5 Run `openspec validate live-part-tinycc-0-9-26` after evidence is recorded.
