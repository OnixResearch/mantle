## Implementation

- [ ] I1 Reproduce the Mes-built TinyCC 0.9.26 immediate-shift bug and record disassembly. [covers=bootstrap.part.tinycc.0.9.26.shift-immediates] [evidence=evidence/I1-reproduction.md]
- [ ] I2 Implement the smallest `bootstrap/tinycc-mes.ncl` codegen repair for immediate shift counts. [covers=bootstrap.part.tinycc.0.9.26.shift-immediates] [evidence=evidence/I2-fix.md]
- [ ] I3 Apply and document any remaining narrow TinyCC 0.9.27 varargs/path patch exposed after the predecessor shift repair, or record that none was needed. [covers=bootstrap.part.tinycc.0.9.26.shift-immediates] [evidence=evidence/I3-tinycc27-followup.md]

## Verification

- [ ] V1 Run source-pin audit for `bootstrap/tinycc-mes.ncl` and `bootstrap/tinycc.ncl`. [covers=bootstrap.part.tinycc.0.9.26.shift-immediates] [evidence=evidence/V1-source-pins.md]
- [ ] V2 Build `bootstrap/tinycc-mes.ncl` and verify the shift reproducer disassembles with nonzero immediate counts (`shr $0x8`, `shl $0x3`, or equivalent). [covers=bootstrap.part.tinycc.0.9.26.shift-immediates] [evidence=evidence/V2-shift-codegen.md]
- [ ] V3 Build `bootstrap/tinycc.ncl` from the repaired predecessor. [covers=bootstrap.part.tinycc.0.9.26.shift-immediates] [evidence=evidence/V3-tinycc27-build.md]
- [ ] V4 Smoke-test TinyCC 0.9.27 version, `hello.c` object compile, GNU make `getopt.c` object compile, and malformed-input clean failure under bounded timeouts; assert `hello.o` and `getopt.o` both exist and are non-empty. [covers=bootstrap.part.tinycc.0.9.26.shift-immediates] [evidence=evidence/V4-smoke.md]
- [ ] V5 Scan `bootstrap/tinycc-mes.ncl`, `bootstrap/tinycc.ncl`, and build logs with `grep -E` for undeclared `/usr/bin|/bin/(cc|gcc|ld|ar)|/lib(64)?/|ld-linux|libc\.so|nix-|nix |/nix/store/.+-(gcc|binutils|glibc)` host-tool, host-libc, host-path, or environment leakage. [covers=bootstrap.part.tinycc.0.9.26.shift-immediates] [evidence=evidence/V5-host-leakage.md]
- [ ] V6 Run OpenSpec validation and tasks gate for this repair. [covers=bootstrap.part.tinycc.0.9.26.shift-immediates] [evidence=evidence/V6-openspec.md]
