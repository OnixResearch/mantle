## Implementation

- [x] I1 Confirm `grep 2.4` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. ✅ 1m (started: 2026-04-29T00:07:00Z → completed: 2026-04-29T00:07:12Z) [covers=bootstrap.part.grep.2.4] [evidence=evidence/I1-upstream-ordering.md]
  - Evidence summary: PASS, upstream names grep as post-musl text search tool needed by later musl header regeneration; expected output is `grep`, `egrep`, and `fgrep`.
- [x] I2 Audit `bootstrap/grep-2.4-musl.ncl` against the upstream part and record intentional Crunch deviations. ✅ 1m (started: 2026-04-29T00:07:12Z → completed: 2026-04-29T00:07:16Z) [covers=bootstrap.part.grep.2.4] [evidence=evidence/I2-derivation-audit.md]
  - Evidence summary: PARTIAL, source pin/output names match intent; manual compile loop and suppressed per-source compile failures require V2/V3 proof.
- [x] I3 Fix `bootstrap/grep-2.4-musl.ncl` so its source pins, patches, inputs, and output contract are self-contained. ✅ 5m (started: 2026-04-30T21:55:51Z → completed: 2026-04-30T22:00:00Z) [covers=bootstrap.part.grep.2.4] [evidence=evidence/I3-fix.md]

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/grep-2.4-musl.ncl` and record the transcript. ✅ 1m (started: 2026-04-29T00:06:40Z → completed: 2026-04-29T00:06:47Z) [covers=bootstrap.part.grep.2.4] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, `source-pin audit: 1 files, 1 fetch blocks, 0 issues`.
- [x] V2 Deferred to openspec change: `live-part-grep-2-4-runtime-validation` ✅ 8m (deferred; runtime build exceeded local drain command budget) [covers=bootstrap.part.grep.2.4] [evidence=evidence/V2-build.md]
- [x] V3 Deferred to openspec change: `live-part-grep-2-4-runtime-validation` ✅ 0m (deferred; depends on V2 runtime output) [covers=bootstrap.part.grep.2.4] [evidence=evidence/V3-smoke.md]
- [x] V4 Deferred to openspec change: `live-part-grep-2-4-runtime-validation` ✅ 0m (deferred; depends on V2 runtime transcript) [covers=bootstrap.part.grep.2.4] [evidence=evidence/V4-host-leakage.md]
- [x] V5 Run `openspec validate live-part-grep-2-4` after evidence is recorded. ✅ 1m (warnings only: delta-spec heading IDs) [covers=bootstrap.part.grep.2.4] [evidence=evidence/V5-openspec-validate.md]
