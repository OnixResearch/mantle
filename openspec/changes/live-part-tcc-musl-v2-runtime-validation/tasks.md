# Tasks: Complete tcc musl v2 runtime validation

## Validation

- [ ] V1 Confirm prerequisite runtime blockers (make 3.82, tcc-musl, rebuilt musl) are resolved or record current blockers. [covers=bootstrap.part.tcc.musl.v2.runtime-validation]
- [ ] V2 Build `bootstrap/tcc-musl-v2.ncl` and record output path or failure class plus provider/fallback/placeholder fields. [covers=bootstrap.part.tcc.musl.v2.runtime-validation]
- [ ] V3 Smoke-test installed `tcc`, `tcc-0.9.27-musl-v2`, `libtcc1.a`, and trivial C compilation. [covers=bootstrap.part.tcc.musl.v2.runtime-validation]
- [ ] V4 Scan derivation and transcript for undeclared host-tool/path/environment leakage. [covers=bootstrap.part.tcc.musl.v2.runtime-validation]
- [ ] V5 Run OpenSpec validation/gates before archive. [covers=bootstrap.part.tcc.musl.v2.runtime-validation]
