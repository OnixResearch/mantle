## Evidence format

Every task records evidence under `openspec/changes/live-part-musl-1-1-24-tcc/evidence/`. Markdown evidence files include `Task-ID:` and `Covers:` metadata. Raw command transcripts and scans are stored beside the markdown evidence as `*-full.log`, `*-output.txt`, or `*-transcript-only.log` files and are referenced from the owning task.

## Implementation

- [x] I1 Confirm `musl 1.1.24 and musl_target` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. ✅ 2m (started: 2026-04-28T23:55:20Z → completed: 2026-04-28T23:55:29Z) [covers=bootstrap.part.musl.1.1.24.tcc] [evidence=evidence/I1-upstream-ordering.md]
- [x] I2 Audit `bootstrap/musl-1.1.24-tcc.ncl` against the upstream part and record intentional Crunch deviations. ✅ 2m (started: 2026-04-28T23:55:29Z → completed: 2026-04-28T23:55:36Z) [covers=bootstrap.part.musl.1.1.24.tcc] [evidence=evidence/I2-derivation-audit.md]
  - Evidence summary: PARTIAL, source/dependency shape matches but upstream source removals and crt-object contract still need I3/V3 proof.
- [ ] I3 Fix `bootstrap/musl-1.1.24-tcc.ncl` so its source pins, patches, inputs, and output contract are self-contained.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/musl-1.1.24-tcc.ncl` and record the transcript. ✅ 1m (started: 2026-04-28T23:55:15Z → completed: 2026-04-28T23:55:20Z) [covers=bootstrap.part.musl.1.1.24.tcc] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, source-pin audit reported `1 files, 1 fetch blocks, 0 issues`.
- [ ] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/musl-1.1.24-tcc.ncl` with the documented bootstrap build environment and record output path plus elapsed time.
- [ ] V3 Smoke-test the produced output contract for `musl 1.1.24 (tcc)`.
- [ ] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage.
- [ ] V5 Run `openspec validate live-part-musl-1-1-24-tcc` after evidence is recorded.
