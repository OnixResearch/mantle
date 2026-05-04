# Tasks: Complete rebuilt musl 1.1.24 runtime validation

## Validation

- [x] V1 Confirm prerequisite runtime blockers (make 3.82, tcc-musl, first musl pass) are resolved or record current blockers. Evidence: `evidence/V1-prerequisite-blockers.md`. [covers=bootstrap.part.musl.1.1.24.tcc.musl.runtime-validation]
- [x] V2 Build `bootstrap/musl-1.1.24-tcc-musl.ncl` and record output path or failure class plus provider/fallback/placeholder fields. Evidence: `evidence/V2-build.md` and `evidence/V2-musl-1.1.24-tcc-musl-pass-*`. [covers=bootstrap.part.musl.1.1.24.tcc.musl.runtime-validation]
- [x] V3 Smoke-test produced `libc.a`, headers, and startup object contract. Evidence: `evidence/V3-smoke.md`. [covers=bootstrap.part.musl.1.1.24.tcc.musl.runtime-validation]
- [x] V4 Scan derivation and transcript for undeclared host-tool/path/environment leakage. Evidence: `evidence/V4-host-leakage.md`. [covers=bootstrap.part.musl.1.1.24.tcc.musl.runtime-validation]
- [x] V5 Run OpenSpec validation/gates before archive. Evidence: `evidence/V5-openspec-validate.md`. [covers=bootstrap.part.musl.1.1.24.tcc.musl.runtime-validation]
