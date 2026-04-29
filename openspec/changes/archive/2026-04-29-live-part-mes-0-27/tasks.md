## Evidence format

Every task records evidence under `openspec/changes/live-part-mes-0-27/evidence/`. Markdown evidence files include `Task-ID:` and `Covers:` metadata. Raw command transcripts and scans are stored beside the markdown evidence as `*-full.log`, `*-output.txt`, or `*-transcript-only.log` files and are referenced from the owning task.

## Implementation

- [x] I1 Confirm `mes 0.27` ordering, source notes, and expected output contract from `parts.rst` and upstream script `steps/mes-0.27.1/pass1.kaem`. ✅ 3m (started: 2026-04-28T23:44:10Z → completed: 2026-04-28T23:44:35Z) [covers=bootstrap.part.mes.0.27] [evidence=evidence/I1-upstream-ordering.md]
- [x] I2 Audit `bootstrap/mes.ncl` against the upstream part and record intentional Crunch deviations. ✅ 3m (started: 2026-04-28T23:44:42Z → completed: 2026-04-28T23:44:55Z) [covers=bootstrap.part.mes.0.27] [evidence=evidence/I2-derivation-audit.md]
- [x] I3 Fix `bootstrap/mes.ncl` so its source pins, patches, inputs, and output contract are self-contained. ✅ 1m (started: 2026-04-28T23:47:20Z → completed: 2026-04-28T23:47:28Z) [covers=bootstrap.part.mes.0.27] [evidence=evidence/I3-self-contained-check.md]
  - Evidence summary: PASS, no source edit required; source pins, explicit inputs, in-derivation patches, and build-time output assertions are self-contained.
- [x] I4 Record direct predecessor compatibility and downstream-blocker locality guardrails. ✅ 1m (started: 2026-04-29T00:03:05Z → completed: 2026-04-29T00:03:12Z) [covers=bootstrap.part.mes.0.27] [evidence=evidence/I4-compatibility-guardrails.md]
  - Evidence summary: PASS, Mes consumes `stage0-posix` as direct predecessor; downstream tinycc/tcc/musl failures are tracked in separate active part changes unless they expose a missing Mes output contract item.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/mes.ncl` and record the transcript. ✅ 1m (started: 2026-04-28T23:45:05Z → completed: 2026-04-28T23:45:15Z) [covers=bootstrap.part.mes.0.27] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, source-pin audit reported `1 files, 2 fetch blocks, 0 issues`.
- [x] V2 Run `/tmp/crunch-build/debug/crunch build bootstrap/mes.ncl` with the documented bootstrap build environment and record output path plus elapsed time. ✅ 15m (started: 2026-04-28T23:44:10Z → completed: 2026-04-28T23:58:36Z) [covers=bootstrap.part.mes.0.27] [evidence=evidence/V2-build.md]
  - Evidence summary: PASS, output `target/live-part-mes-0-27/store/anqp2lm1qgppznmndhgj7ighp8fbx6wn-mes`, hermeticity `practical (no degraded facts)`.
- [x] V3 Smoke-test the produced output contract for `mes 0.27`. ✅ 2m (started: 2026-04-28T23:59:16Z → completed: 2026-04-29T00:00:13Z) [covers=bootstrap.part.mes.0.27] [evidence=evidence/V3-smoke.md]
  - Evidence summary: PASS, required binaries/libs/headers/modules exist, `mes-m2` evaluates a Scheme smoke, mescc help reports `C99 compiler in Scheme`, and mescc compiles a trivial C program to M1 output containing `:main`.
- [x] V4 Scan the derivation and log for undeclared host-tool, path, or environment leakage. ✅ 1m (started: 2026-04-29T00:00:45Z → completed: 2026-04-29T00:01:05Z) [covers=bootstrap.part.mes.0.27] [evidence=evidence/V4-host-leakage.md]
  - Evidence summary: PASS, findings are limited to declared `/bin/sh`/`/bin/busybox`, output-internal paths, final output path, source URLs, comments, and post-build smoke harness paths.
- [x] V5 Run `openspec validate live-part-mes-0-27` after evidence is recorded. ✅ 1m (started: 2026-04-29T00:01:08Z → completed: 2026-04-29T00:01:10Z) [covers=bootstrap.part.mes.0.27] [evidence=evidence/V5-openspec-validate.md]
  - Evidence summary: PASS, `Change 'live-part-mes-0-27' is valid`.
