## Implementation

- [ ] I1 Reproduce and localize the `tinycc 0.9.27` amd64 compile hang with bounded `tcc -c` probes. [covers=bootstrap.part.tinycc.0.9.27.amd64.compile] [evidence=evidence/I1-reproduction.md]
- [ ] I2 Implement the smallest stable `bootstrap/tinycc.ncl` repair for amd64 object compilation. [covers=bootstrap.part.tinycc.0.9.27.amd64.compile] [evidence=evidence/I2-fix.md]

## Verification

- [ ] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/tinycc.ncl`. [covers=bootstrap.part.tinycc.0.9.27.amd64.compile] [evidence=evidence/V1-source-pins.md]
- [ ] V2 Run `crunch build bootstrap/tinycc.ncl` with the documented bootstrap environment. [covers=bootstrap.part.tinycc.0.9.27.amd64.compile] [evidence=evidence/V2-build.md]
- [ ] V3 Smoke-test version, trivial object compilation, and malformed-input clean failure: run `bin/tcc -version`; run `timeout 5s bin/tcc -c hello.c -o hello.o` and assert exit 0 plus non-empty `hello.o`; run `timeout 5s bin/tcc -c malformed.c -o malformed.o` and assert nonzero, not timeout-derived (`124`), and not signal-derived/segfault (`139`). [covers=bootstrap.part.tinycc.0.9.27.amd64.compile] [evidence=evidence/V3-smoke.md]
- [ ] V4 Scan `bootstrap/tinycc.ncl` and the V2 build log with `grep -E` for undeclared `/usr/bin|/bin/(cc|gcc|ld|ar|sh)|nix-|nix |/nix/store/.+-(gcc|binutils|glibc)` host-tool, host-path, or environment leakage, allowing declared stage0/mes/tinycc inputs only. [covers=bootstrap.part.tinycc.0.9.27.amd64.compile] [evidence=evidence/V4-host-leakage.md]
- [ ] V5 Run OpenSpec validation and tasks gate for this repair change. [covers=bootstrap.part.tinycc.0.9.27.amd64.compile] [evidence=evidence/V5-openspec.md]
