# Tasks: Complete tcc musl prep runtime validation

## Validation

- [ ] V1 Confirm prerequisite runtime blockers (make 3.82 and TinyCC amd64 repair) are resolved or record current blockers. [covers=bootstrap.part.tcc.musl.prep.runtime-validation]
- [ ] V2 Build `bootstrap/tcc-musl-prep.ncl` and record output path or failure class plus provider/fallback/placeholder fields. [covers=bootstrap.part.tcc.musl.prep.runtime-validation]
- [ ] V3 Smoke-test installed `tcc`, `tcc-musl-prep`, carried Mes libc/headers, and `tcc -v`. [covers=bootstrap.part.tcc.musl.prep.runtime-validation]
- [ ] V4 Scan derivation and transcript for undeclared host-tool/path/environment leakage. [covers=bootstrap.part.tcc.musl.prep.runtime-validation]
- [ ] V5 Run OpenSpec validation/gates before archive. [covers=bootstrap.part.tcc.musl.prep.runtime-validation]
