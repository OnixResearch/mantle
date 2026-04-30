## Implementation

- [x] I1 Confirm `make 3.82` ordering, source notes, and expected output contract from `parts.rst` and the matching `steps/` script. ✅ 1m (started: 2026-04-29T00:13:20Z → completed: 2026-04-29T00:13:42Z) [covers=bootstrap.part.make.3.82] [evidence=evidence/I1-upstream-ordering.md]
  - Evidence summary: PASS, this is the early tcc-built make; expected output is `bin/make` reporting GNU Make 3.82.
- [x] I2 Audit `bootstrap/make-tcc.ncl` against the upstream part and record intentional Crunch deviations. ✅ 1m (started: 2026-04-29T00:13:42Z → completed: 2026-04-29T00:13:47Z) [covers=bootstrap.part.make.3.82] [evidence=evidence/I2-derivation-audit.md]
  - Evidence summary: PARTIAL, source pin and source/object list match upstream pass1; build/smoke proof still required.
- [x] I3 Deferred `bootstrap/make-tcc.ncl` amd64 execution repair to openspec change `repair-make-tcc-amd64-varargs`. ✅ 0m (deferred) [covers=bootstrap.part.make.3.82] [evidence=evidence/I3-deferred.md]
  - Evidence summary: DEFERRED, Mes-linked `tinycc 0.9.27` hangs compiling C and the `tinycc 0.9.26` fallback exposes GNU make varargs/string-construction crashes on amd64.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/make-tcc.ncl` and record the transcript. ✅ 1m (started: 2026-04-29T00:13:35Z → completed: 2026-04-29T00:13:38Z) [covers=bootstrap.part.make.3.82] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, `source-pin audit: 1 files, 1 fetch blocks, 0 issues`.
- [x] V2 Deferred to openspec change: `live-part-make-3-82-runtime-validation` ✅ 0m (deferred; blocked by `repair-make-tcc-amd64-varargs`) [covers=bootstrap.part.make.3.82] [evidence=evidence/V2-build.md]
  - Blocked by `repair-make-tcc-amd64-varargs`; diagnostic build transcripts are preserved in `evidence/V2-build-full.log` but are not PASS evidence.
- [x] V3 Deferred to openspec change: `live-part-make-3-82-runtime-validation` ✅ 0m (deferred; version output alone is insufficient) [covers=bootstrap.part.make.3.82] [evidence=evidence/V3-smoke.md]
  - Blocked by `repair-make-tcc-amd64-varargs`; diagnostic smoke transcript proves version output but simple Makefile execution still segfaults.
- [x] V4 Deferred to openspec change: `live-part-make-3-82-runtime-validation` ✅ 0m (deferred; depends on repaired runtime transcript) [covers=bootstrap.part.make.3.82] [evidence=evidence/V4-host-leakage.md]
  - Blocked until the repaired derivation exists in `repair-make-tcc-amd64-varargs`.
- [x] V5 Run `openspec validate live-part-make-3-82` after evidence is recorded. ✅ 1m (warnings only: delta-spec heading IDs) [covers=bootstrap.part.make.3.82] [evidence=evidence/V5-openspec-validate.md]
