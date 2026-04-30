## Implementation

- [x] I1 Reproduce and minimize the amd64 make crash using the saved parent evidence and a tiny Makefile. ✅ 25m (started: 2026-04-30T15:08:00Z → completed: 2026-04-30T15:33:00Z) [covers=bootstrap.part.make.3.82.amd64.execution] [evidence=evidence/I1-reproduction.md]
  - Evidence summary: PASS, `tinycc 0.9.27` hangs compiling C; `tinycc 0.9.26` builds the object list but GNU make fails on amd64 runtime paths (`concat(...)` segfault first, then status/recipe execution failure after fixed-arity concat experiments).
- [x] I2 Deferred predecessor compiler repair to `repair-tinycc-0-9-27-amd64-compile`. ✅ 0m (deferred: 2026-04-30T15:48:00Z) [covers=bootstrap.part.make.3.82.amd64.execution] [evidence=evidence/I2-deferred-to-tinycc.md]
  - Evidence summary: DEFERRED, make-local patches plus `tinycc 0.9.26` still leave GNU make exiting `60` without recipe execution; direct `tinycc 0.9.27` hangs compiling C, so predecessor repair is required before make can pass smoke evidence.
- [x] I3 Deferred TinyCC immediate-shift repair to `repair-tinycc-mes-shift-immediates`. ✅ 10m (started: 2026-04-30T16:26:45Z → deferred: 2026-04-30T16:36:30Z) [covers=bootstrap.part.make.3.82.amd64.execution] [evidence=evidence/I3-deferred-to-shift-immediates.md]
  - Evidence summary: DEFERRED, repaired TinyCC 0.9.27 compiles trivial C but GNU make `getopt.c` still exposes predecessor `tinycc 0.9.26` constant-shift codegen (`shr $0x0`, `shl $0x0`).

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/make-tcc.ncl`. ✅ 1m (started: 2026-04-30T23:04:00Z → completed: 2026-04-30T23:05:00Z) [covers=bootstrap.part.make.3.82.amd64.execution] [evidence=evidence/V1-source-pins.md]
- [ ] V2 Run `crunch build bootstrap/make-tcc.ncl` with the documented bootstrap environment. [covers=bootstrap.part.make.3.82.amd64.execution] [evidence=evidence/V2-build.md]
- [ ] V3 Smoke-test version, simple Makefile success, and missing-target clean failure. [covers=bootstrap.part.make.3.82.amd64.execution] [evidence=evidence/V3-smoke.md]
- [ ] V4 Scan the derivation and build logs for undeclared host-tool, host-path, and environment leakage. [covers=bootstrap.part.make.3.82.amd64.execution] [evidence=evidence/V4-host-leakage.md]
- [ ] V5 Run OpenSpec validation and tasks gate for this repair change. [covers=bootstrap.part.make.3.82.amd64.execution] [evidence=evidence/V5-openspec.md]
