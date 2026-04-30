## Implementation

- [x] I1 Reproduce the Mes-built TinyCC 0.9.26 immediate-shift bug and record disassembly. ✅ 1m (started: 2026-04-30T16:40:00Z → completed: 2026-04-30T16:40:19Z) [covers=bootstrap.part.tinycc.0.9.26.shift-immediates] [evidence=evidence/I1-reproduction.md]
  - Evidence summary: PASS, `x >> 8` disassembles as `shr $0x0,%eax` and `x << 3` as `shl $0x0,%eax`, while `x & 31` correctly emits `and $0x1f,%eax`.
- [x] I2 Implement the smallest `bootstrap/tinycc-mes.ncl` codegen repair for immediate shift counts. ✅ 10m (started: 2026-04-30T16:40:45Z → completed: 2026-04-30T16:50:43Z) [covers=bootstrap.part.tinycc.0.9.26.shift-immediates] [evidence=evidence/I2-fix.md]
  - Evidence summary: PASS, `bootstrap/tinycc-mes.ncl` rewrites the x86_64 immediate-shift count emission from the Mes-miscompiled ternary mask expression to direct byte emission, and pueue task 28 rebuilt TinyCC 0.9.26 successfully.
- [x] I3 Apply and document any remaining narrow TinyCC 0.9.27 varargs/path patch exposed after the predecessor shift repair, or record that none was needed. ✅ 10m (started: 2026-04-30T16:51:50Z → completed: 2026-04-30T17:01:08Z) [covers=bootstrap.part.tinycc.0.9.26.shift-immediates] [evidence=evidence/I3-tinycc27-followup.md]
  - Evidence summary: PASS, TinyCC 0.9.27 needed one additional direct-string `tccelf.c` relocation-section-name patch to avoid Mes varargs `snprintf` corruption during global-reference/getopt compilation.

## Verification

- [x] V1 Run source-pin audit for `bootstrap/tinycc-mes.ncl` and `bootstrap/tinycc.ncl`. ✅ 1m (started: 2026-04-30T16:50:29Z → completed: 2026-04-30T16:50:39Z) [covers=bootstrap.part.tinycc.0.9.26.shift-immediates] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, `source-pin audit: 2 files, 3 fetch blocks, 0 issues`.
- [x] V2 Build `bootstrap/tinycc-mes.ncl` and verify the shift reproducer disassembles with nonzero immediate counts (`shr $0x8`, `shl $0x3`, or equivalent). ✅ 8m (started: 2026-04-30T16:41:12Z → completed: 2026-04-30T16:49:31Z) [covers=bootstrap.part.tinycc.0.9.26.shift-immediates] [evidence=evidence/V2-shift-codegen.md]
  - Evidence summary: PASS, rebuilt TinyCC 0.9.26 emits `sar $0x8,%eax`, `shl $0x3,%eax`, and `and $0x1f,%eax` for the reproducer.
- [x] V3 Build `bootstrap/tinycc.ncl` from the repaired predecessor. ✅ 1m (started: 2026-04-30T17:00:16Z → completed: 2026-04-30T17:00:17Z) [covers=bootstrap.part.tinycc.0.9.26.shift-immediates] [evidence=evidence/V3-tinycc27-build.md]
  - Evidence summary: PASS, `tinycc-0.9.27.drv` built to `/home/brittonr/git/crunch/crunch/target/live-part-tinycc-0-9-27/run-current/store/44mb0vkwi7i3cs5r5dj8l92645a250g6-tinycc-0.9.27` with `hermeticity: practical (no degraded facts)`.
- [x] V4 Smoke-test TinyCC 0.9.27 version, `hello.c` object compile, GNU make `getopt.c` object compile, and malformed-input clean failure under bounded timeouts; assert `hello.o` and `getopt.o` both exist and are non-empty. ✅ 1m (started: 2026-04-30T17:00:17Z → completed: 2026-04-30T17:00:28Z) [covers=bootstrap.part.tinycc.0.9.26.shift-immediates] [evidence=evidence/V4-smoke.md]
  - Evidence summary: PASS, version reported `0.9.27`, `hello.o` size was 672 bytes, `getopt.o` size was 15516 bytes, and malformed input exited 1 instead of timeout/segfault.
- [x] V5 Scan `bootstrap/tinycc-mes.ncl`, `bootstrap/tinycc.ncl`, and build logs with `grep -E` for undeclared `/usr/bin|/bin/(cc|gcc|ld|ar)|/lib(64)?/|ld-linux|libc\.so|nix-|nix |/nix/store/.+-(gcc|binutils|glibc)` host-tool, host-libc, host-path, or environment leakage. ✅ 1m (started: 2026-04-30T17:01:43Z → completed: 2026-04-30T17:02:01Z) [covers=bootstrap.part.tinycc.0.9.26.shift-immediates] [evidence=evidence/V5-host-leakage.md]
  - Evidence summary: PASS, grep hits were reviewed as declared `$out/lib/mes`, `$PREFIX/lib/mes`, or predecessor Mes runtime paths; no undeclared host tool/libc/path match was present.
- [x] V6 Run OpenSpec validation and tasks gate for this repair. ✅ 1m (started: 2026-04-30T17:02:21Z → completed: 2026-04-30T17:02:52Z) [covers=bootstrap.part.tinycc.0.9.26.shift-immediates] [evidence=evidence/V6-openspec.md]
  - Evidence summary: PASS, `openspec validate repair-tinycc-mes-shift-immediates --strict` passed and the tasks gate passed after V6 evidence was recorded.
