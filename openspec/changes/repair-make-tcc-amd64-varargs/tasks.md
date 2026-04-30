## Implementation

- [ ] I1 Reproduce and minimize the amd64 make crash using the saved parent evidence and a tiny Makefile. [covers=bootstrap.part.make.3.82.amd64.execution] [evidence=evidence/I1-reproduction.md]
- [ ] I2 Implement the smallest stable `bootstrap/make-tcc.ncl` repair for amd64 execution. [covers=bootstrap.part.make.3.82.amd64.execution] [evidence=evidence/I2-fix.md]

## Verification

- [ ] V1 Run `cargo -Zscript scripts/check-bootstrap-source-pins.rs bootstrap/make-tcc.ncl`. [covers=bootstrap.part.make.3.82.amd64.execution] [evidence=evidence/V1-source-pins.md]
- [ ] V2 Run `crunch build bootstrap/make-tcc.ncl` with the documented bootstrap environment. [covers=bootstrap.part.make.3.82.amd64.execution] [evidence=evidence/V2-build.md]
- [ ] V3 Smoke-test version, simple Makefile success, and missing-target clean failure. [covers=bootstrap.part.make.3.82.amd64.execution] [evidence=evidence/V3-smoke.md]
- [ ] V4 Scan the derivation and build logs for undeclared host-tool, host-path, and environment leakage. [covers=bootstrap.part.make.3.82.amd64.execution] [evidence=evidence/V4-host-leakage.md]
- [ ] V5 Run OpenSpec validation and tasks gate for this repair change. [covers=bootstrap.part.make.3.82.amd64.execution] [evidence=evidence/V5-openspec.md]
