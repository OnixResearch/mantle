# Tasks: Complete make 3.82 runtime validation

## Validation

- [x] V1 Confirm `repair-make-tcc-amd64-varargs` is complete or record its current blocker. [covers=bootstrap.part.make.3.82.runtime-validation] Evidence: `evidence/V1-prerequisite-status.md`.
- [x] V2 Build `bootstrap/make-tcc.ncl` and record output path or failure class plus provider/fallback/placeholder fields. [covers=bootstrap.part.make.3.82.runtime-validation] Evidence: `evidence/V2-build-attempt.md` (`incomplete-hung-validation-run`).
- [ ] V3 Smoke-test `make --version` and simple Makefile execution with the produced GNU Make 3.82 output. [covers=bootstrap.part.make.3.82.runtime-validation]
- [ ] V4 Scan derivation and transcript for undeclared host-tool/path/environment leakage. [covers=bootstrap.part.make.3.82.runtime-validation]
- [ ] V5 Run OpenSpec validation/gates before archive. [covers=bootstrap.part.make.3.82.runtime-validation]
