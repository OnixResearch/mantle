# Tasks: Complete tcc musl runtime validation

## Validation

- [ ] V1 Confirm prerequisite runtime blockers (make 3.82, first musl pass, tcc-musl-prep) are resolved or record current blockers. [covers=bootstrap.part.tcc.musl.runtime-validation]
- [ ] V2 Build `bootstrap/tcc-musl.ncl` and record output path or failure class plus provider/fallback/placeholder fields. [covers=bootstrap.part.tcc.musl.runtime-validation]
- [ ] V3 Smoke-test installed `tcc`, `tcc-0.9.27-musl`, `libtcc1.a`, and trivial C compilation. [covers=bootstrap.part.tcc.musl.runtime-validation]
- [ ] V4 Scan derivation and transcript for undeclared host-tool/path/environment leakage. [covers=bootstrap.part.tcc.musl.runtime-validation]
- [ ] V5 Run OpenSpec validation/gates before archive. [covers=bootstrap.part.tcc.musl.runtime-validation]
