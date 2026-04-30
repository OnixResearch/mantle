## Implementation

- [x] I1 Reproduce and localize the `tinycc 0.9.27` amd64 compile hang with bounded `tcc -c` probes. ✅ 1m (started: 2026-04-30T15:50:40Z → completed: 2026-04-30T15:51:20Z) [covers=bootstrap.part.tinycc.0.9.27.amd64.compile] [evidence=evidence/I1-reproduction.md]
  - Evidence summary: PASS, version exits 0 but `tcc -c hello.c` times out with no object after repeated `brk()` growth; verbose and malformed-input probes segfault through Mes-libc-corrupted diagnostics.
- [x] I2 Implement the smallest stable `bootstrap/tinycc.ncl` repair for amd64 object compilation. ✅ 29m (started: 2026-04-30T15:51:30Z → completed: 2026-04-30T16:20:24Z) [covers=bootstrap.part.tinycc.0.9.27.amd64.compile] [evidence=evidence/I2-fix.md]
  - Evidence summary: PASS, tcc-0.9.26 emitted `shr $0` for TinyCC 0.9.27 byte-emission shifts; `bootstrap/tinycc.ncl` now patches x86_64 byte emitters/REX helpers to avoid power-of-two shifts and patches error cleanup so malformed input exits cleanly.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tinycc.ncl`. ✅ 1m (started: 2026-04-30T16:18:35Z → completed: 2026-04-30T16:19:48Z) [covers=bootstrap.part.tinycc.0.9.27.amd64.compile] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, source-pin audit reports `1 files, 1 fetch blocks, 0 issues`.
- [x] V2 Run `crunch build bootstrap/tinycc.ncl` with the documented bootstrap environment. ✅ 1m (started: 2026-04-30T16:19:50Z → completed: 2026-04-30T16:19:59Z) [covers=bootstrap.part.tinycc.0.9.27.amd64.compile] [evidence=evidence/V2-build.md]
  - Evidence summary: PASS, output `/home/brittonr/git/crunch/crunch/target/live-part-tinycc-0-9-27/run-current/store/aqq1ifqcckz0nllskfqz3sz36mz23n8i-tinycc-0.9.27`; hermeticity practical with no degraded facts.
- [x] V3 Smoke-test version, trivial object compilation, and malformed-input clean failure: run `bin/tcc -version`; run `timeout 5s bin/tcc -c hello.c -o hello.o` and assert exit 0 plus non-empty `hello.o`; run `timeout 5s bin/tcc -c malformed.c -o malformed.o` and assert nonzero, not timeout-derived (`124`), and not signal-derived/segfault (`139`). ✅ 1m (started: 2026-04-30T16:20:00Z → completed: 2026-04-30T16:20:07Z) [covers=bootstrap.part.tinycc.0.9.27.amd64.compile] [evidence=evidence/V3-smoke.md]
  - Evidence summary: PASS, version rc 0, `hello.o` non-empty, malformed rc 1, no timeout or segfault.
- [x] V4 Scan `bootstrap/tinycc.ncl` and the V2 build log with `grep -E` for undeclared `/usr/bin|/bin/(cc|gcc|ld|ar|sh)|nix-|nix |/nix/store/.+-(gcc|binutils|glibc)` host-tool, host-path, or environment leakage, allowing declared stage0/mes/tinycc inputs only. ✅ 1m (started: 2026-04-30T16:20:10Z → completed: 2026-04-30T16:20:16Z) [covers=bootstrap.part.tinycc.0.9.27.amd64.compile] [evidence=evidence/V4-host-leakage.md]
  - Evidence summary: PASS, leakage grep rc 1 and `host_leakage=none`.
- [x] V5 Run OpenSpec validation and tasks gate for this repair change. ✅ 2m (started: 2026-04-30T16:21:10Z → completed: 2026-04-30T16:22:35Z) [covers=bootstrap.part.tinycc.0.9.27.amd64.compile] [evidence=evidence/V5-openspec.md]
  - Evidence summary: PASS, `openspec validate repair-tinycc-0-9-27-amd64-compile --strict` passed; final tasks gate passed after V5 was marked complete.
