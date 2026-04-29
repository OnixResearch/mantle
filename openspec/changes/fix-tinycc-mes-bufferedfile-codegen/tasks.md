## Implementation

- [ ] I1 Reproduce and minimize the `BufferedFile` diagnostics from `bootstrap/tinycc-mes.ncl`.
- [ ] I2 Compare Crunch `bootstrap/tinycc-mes.ncl` against upstream `steps/tcc-0.9.26/pass1.kaem` and required simple patches.
- [ ] I3 Fix the smallest derivation/source-normalization issue that makes `tcc-boot0` compile without segfaulting.

## Verification

- [ ] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tinycc-mes.ncl`.
- [ ] V2 Run `crunch build bootstrap/tinycc-mes.ncl --no-substitute -j 1 --verbose --log-level info` and record successful output path.
- [ ] V3 Smoke-test produced `bin/tcc` by checking `--version` and compiling a trivial C program.
- [ ] V4 Run `openspec validate fix-tinycc-mes-bufferedfile-codegen`.
