## Runtime Validation

- [ ] V1 Run long-budget `crunch build bootstrap/make-tcc.ncl` with fresh local store/state and preserve the full transcript. [covers=bootstrap.part.make.3.82.amd64.runtime-validation] [evidence=evidence/V1-build.md]
- [ ] V2 Smoke-test the produced make with `--version`, a simple Makefile success, and missing-target clean failure. [covers=bootstrap.part.make.3.82.amd64.runtime-validation] [evidence=evidence/V2-smoke.md]
- [ ] V3 Scan the derivation, build transcript, and output for undeclared host-tool, host-path, and environment leakage. [covers=bootstrap.part.make.3.82.amd64.runtime-validation] [evidence=evidence/V3-host-leakage.md]
- [ ] V4 Run OpenSpec validation for this runtime-validation successor. [covers=bootstrap.part.make.3.82.amd64.runtime-validation] [evidence=evidence/V4-openspec.md]
