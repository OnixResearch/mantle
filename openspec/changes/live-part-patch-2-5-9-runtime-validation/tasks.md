# Tasks: Complete patch 2.5.9 runtime validation

## Validation

- [ ] V1 Confirm prerequisite runtime blockers (make 3.82 and TinyCC amd64 repair) are resolved or record current blockers. [covers=bootstrap.part.patch.2.5.9.runtime-validation]
- [ ] V2 Build `bootstrap/patch-tcc.ncl` and record output path or failure class plus provider/fallback/placeholder fields. [covers=bootstrap.part.patch.2.5.9.runtime-validation]
- [ ] V3 Smoke-test produced `patch` with `--version` and simple unified-diff application. [covers=bootstrap.part.patch.2.5.9.runtime-validation]
- [ ] V4 Scan derivation and transcript for undeclared host-tool/path/environment leakage. [covers=bootstrap.part.patch.2.5.9.runtime-validation]
- [ ] V5 Run OpenSpec validation/gates before archive. [covers=bootstrap.part.patch.2.5.9.runtime-validation]
