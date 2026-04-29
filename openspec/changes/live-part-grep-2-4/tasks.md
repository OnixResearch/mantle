## Implementation

- [x] I1 Confirm `grep 2.4` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. ✅ 1m (started: 2026-04-29T00:07:00Z → completed: 2026-04-29T00:07:12Z) [covers=bootstrap.part.grep.2.4] [evidence=evidence/I1-upstream-ordering.md]
  - Evidence summary: PASS, upstream names grep as post-musl text search tool needed by later musl header regeneration; expected output is `grep`, `egrep`, and `fgrep`.
- [x] I2 Audit `bootstrap/grep-2.4-musl.ncl` against the upstream part and record intentional Crunch deviations. ✅ 1m (started: 2026-04-29T00:07:12Z → completed: 2026-04-29T00:07:16Z) [covers=bootstrap.part.grep.2.4] [evidence=evidence/I2-derivation-audit.md]
  - Evidence summary: PARTIAL, source pin/output names match intent; manual compile loop and suppressed per-source compile failures require V2/V3 proof.
- [ ] I3 Fix `bootstrap/grep-2.4-musl.ncl` so its source pins, patches, inputs, and output contract are self-contained.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/grep-2.4-musl.ncl` and record the transcript. ✅ 1m (started: 2026-04-29T00:06:40Z → completed: 2026-04-29T00:06:47Z) [covers=bootstrap.part.grep.2.4] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, `source-pin audit: 1 files, 1 fetch blocks, 0 issues`.
- [ ] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/grep-2.4-musl.ncl` with the documented bootstrap build environment and record output path plus elapsed time.
- [ ] V3 Smoke-test the produced output contract for `grep 2.4`.
- [ ] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage.
- [ ] V5 Run `openspec validate live-part-grep-2-4` after evidence is recorded.
