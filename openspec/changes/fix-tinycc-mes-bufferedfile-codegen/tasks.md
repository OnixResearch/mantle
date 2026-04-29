## Implementation

- [x] I1 Reproduce and minimize the `BufferedFile` diagnostics from `bootstrap/tinycc-mes.ncl`; record why `tcc-mes -version` remains insufficient evidence. ✅ 1m (started: 2026-04-29T00:28:20Z → completed: 2026-04-29T00:28:31Z) [covers=bootstrap.part.tinycc.0.9.26.selfcompile] [evidence=evidence/I1-reproduction.md]
  - Evidence summary: PASS, failure is minimized to `tcc-boot0` segfault after `BufferedFile` diagnostics; `tcc-mes -version` succeeds before failure and is insufficient.
- [x] I2 Compare Crunch `bootstrap/tinycc-mes.ncl` against upstream `steps/tcc-0.9.26/pass1.kaem` and required simple patches; classify the defect as missing upstream patch, local source normalization, or Mes/mescc setup. ✅ 1m (started: 2026-04-29T00:29:00Z → completed: 2026-04-29T00:29:15Z) [covers=bootstrap.part.tinycc.0.9.26.selfcompile] [evidence=evidence/I2-upstream-comparison.md]
  - Evidence summary: PASS, upstream patch bucket is covered by Crunch `tcctools.c` patch; Mes setup reaches runnable `tcc-mes`; leading defect bucket is local pass1 runtime-refresh parity plus `BufferedFile` declaration cleanup.
- [ ] I3 Fix the smallest classified derivation/source-normalization/Mes setup issue that makes `tcc-boot0` compile without segfaulting.

## Verification

- [x] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tinycc-mes.ncl`. ✅ 1m (started: 2026-04-29T01:20:15Z → completed: 2026-04-29T01:20:20Z) [covers=bootstrap.part.tinycc.0.9.26.selfcompile] [evidence=evidence/V1-source-pins.md]
  - Evidence summary: PASS, `bootstrap/tinycc-mes.ncl` and touched predecessor `bootstrap/mes.ncl` both report source-pin audits with 1 file, 2 fetch blocks, 0 issues.
- [ ] V2 Run `crunch build bootstrap/tinycc-mes.ncl --no-substitute -j 1 --verbose --log-level info` and record successful output path, `bin/tcc`, `bin/tcc-0.9.26`, and no `Segmentation fault` during `tcc-boot0`.
- [ ] V3 Smoke-test produced `bin/tcc` and `bin/tcc-0.9.26` by checking `--version` and compiling a trivial C program; explicitly reject `tcc-mes -version` alone as acceptance evidence.
- [ ] V4 Run `openspec validate fix-tinycc-mes-bufferedfile-codegen`.
