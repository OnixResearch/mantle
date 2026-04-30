## Evidence format

Every task records evidence under `openspec/changes/live-part-tcc-musl-v2/evidence/`. Markdown evidence files include `Task-ID:` and `Covers:` metadata. Raw command transcripts and scans are stored beside the markdown evidence as `*-full.log`, `*-output.txt`, or `*-transcript-only.log` files and are referenced from the owning task.

## Implementation

- [x] I1 Confirm `musl 1.1.24 and musl_target` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. ✅ 2m (started: 2026-04-28T23:58:20Z → completed: 2026-04-28T23:58:34Z) [covers=bootstrap.part.tcc.musl.v2] [evidence=evidence/I1-upstream-ordering.md]
- [x] I2 Audit `bootstrap/tcc-musl-v2.ncl` against the upstream part and record intentional Crunch deviations. ✅ 2m (started: 2026-04-28T23:58:34Z → completed: 2026-04-28T23:58:40Z) [covers=bootstrap.part.tcc.musl.v2] [evidence=evidence/I2-derivation-audit.md]
  - Evidence summary: PARTIAL, self-host step exists but `libtcc1.a` output contract remains conditional.
- [x] I3 Fix `bootstrap/tcc-musl-v2.ncl` so its source pins, patches, inputs, and output contract are self-contained. ✅ 5m (started: 2026-04-30T22:53:00Z → completed: 2026-04-30T22:58:00Z) [covers=bootstrap.part.tcc.musl.v2] [evidence=evidence/I3-fix.md]

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tcc-musl-v2.ncl` and record the transcript. ✅ 1m (started: 2026-04-28T23:58:25Z → completed: 2026-04-28T23:58:29Z) [covers=bootstrap.part.tcc.musl.v2] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, source-pin audit reported `1 files, 1 fetch blocks, 0 issues`.
- [ ] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/tcc-musl-v2.ncl` with the documented bootstrap build environment and record output path plus elapsed time.
- [ ] V3 Smoke-test the produced output contract for `tcc musl v2`.
- [ ] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage.
- [ ] V5 Run `openspec validate live-part-tcc-musl-v2` after evidence is recorded.
