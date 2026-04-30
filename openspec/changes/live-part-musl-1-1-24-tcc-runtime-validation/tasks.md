# Tasks: Complete musl 1.1.24 tcc runtime validation

## Validation

- [ ] V1 Confirm prerequisite runtime blockers (`repair-make-tcc-amd64-varargs` and make 3.82 runtime validation) are resolved or record current blockers. [covers=bootstrap.part.musl.1.1.24.tcc.runtime-validation]
- [ ] V2 Build `bootstrap/musl-1.1.24-tcc.ncl` and record output path or failure class plus provider/fallback/placeholder fields. [covers=bootstrap.part.musl.1.1.24.tcc.runtime-validation]
- [ ] V3 Smoke-test produced `libc.a`, headers, and startup object contract. [covers=bootstrap.part.musl.1.1.24.tcc.runtime-validation]
- [ ] V4 Scan derivation and transcript for undeclared host-tool/path/environment leakage. [covers=bootstrap.part.musl.1.1.24.tcc.runtime-validation]
- [ ] V5 Run OpenSpec validation/gates before archive. [covers=bootstrap.part.musl.1.1.24.tcc.runtime-validation]
