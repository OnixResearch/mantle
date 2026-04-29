## Evidence format

Every task records evidence under `openspec/changes/live-part-tinycc-0-9-27/evidence/`. Markdown evidence files include `Task-ID:` and `Covers:` metadata. Raw command transcripts and scans are stored beside the markdown evidence as `*-full.log`, `*-output.txt`, or `*-transcript-only.log` files and are referenced from the owning task.

## Implementation

- [x] I1 Confirm `tinycc 0.9.27` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. ✅ 2m (started: 2026-04-28T23:50:03Z → completed: 2026-04-28T23:50:17Z) [covers=bootstrap.part.tinycc.0.9.27] [evidence=evidence/I1-upstream-ordering.md]
- [x] I2 Audit `bootstrap/tinycc.ncl` against the upstream part and record intentional Crunch deviations. ✅ 2m (started: 2026-04-28T23:50:17Z → completed: 2026-04-28T23:50:23Z) [covers=bootstrap.part.tinycc.0.9.27] [evidence=evidence/I2-derivation-audit.md]
  - Evidence summary: PARTIAL, dependency/source order matched; I3 applies the missing upstream source patches and records the amd64 Mes runtime rebuild as an intentional downstream handoff.
- [x] I3 Fix `bootstrap/tinycc.ncl` so its source pins, patches, inputs, and output contract are self-contained. ✅ 1h (started: 2026-04-29T17:35:10Z → completed: 2026-04-29T18:33:23Z) [covers=bootstrap.part.tinycc.0.9.27] [evidence=evidence/I3-fix.md]
  - Evidence summary: PASS, derivation applies upstream tcc 0.9.27 patches, predecessor-safe source normalizations, Mes-runtime string-format workarounds, and installs the expected `bin/`, `include/mes`, and `lib/mes` contract.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tinycc.ncl` and record the transcript. ✅ 1m (started: 2026-04-28T23:50:10Z → completed: 2026-04-29T18:28:32Z) [covers=bootstrap.part.tinycc.0.9.27] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, source-pin audit reported `1 files, 1 fetch blocks, 0 issues`.
- [x] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/tinycc.ncl` with the documented bootstrap build environment and record output path plus elapsed time. ✅ 1m (started: 2026-04-29T18:32:08Z → completed: 2026-04-29T18:32:08Z) [covers=bootstrap.part.tinycc.0.9.27] [evidence=evidence/V2-build-success.md]
  - Evidence summary: PASS, build returned `/home/brittonr/git/crunch/crunch/target/live-part-tinycc-0-9-27/run-current/store/0vz94d750q8wbylhln3sza1zwcwsvb1p-tinycc-0.9.27` with `hermeticity: practical (no degraded facts)`.
- [x] V3 Smoke-test the produced output contract for `tinycc 0.9.27`. ✅ 1m (started: 2026-04-29T18:32:17Z → completed: 2026-04-29T18:32:17Z) [covers=bootstrap.part.tinycc.0.9.27] [evidence=evidence/V3-smoke.md]
  - Evidence summary: PASS, both compiler entrypoints report `tcc version 0.9.27`, expected runtime/header files exist, and embedded runtime paths point at the produced logical store output.
- [x] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage. ✅ 1m (started: 2026-04-29T18:32:30Z → completed: 2026-04-29T18:32:30Z) [covers=bootstrap.part.tinycc.0.9.27] [evidence=evidence/V4-host-leakage.md]
  - Evidence summary: PASS, no absolute host paths or unqualified shell utilities were found in the derivation body; build reported no degraded hermeticity facts.
- [x] V5 Run `openspec validate live-part-tinycc-0-9-27` after evidence is recorded. ✅ 1m (started: 2026-04-29T18:33:19Z → completed: 2026-04-29T18:33:19Z) [covers=bootstrap.part.tinycc.0.9.27] [evidence=evidence/V5-openspec-validate.md]
  - Evidence summary: PASS, `openspec validate live-part-tinycc-0-9-27 --strict` reported the change valid.
