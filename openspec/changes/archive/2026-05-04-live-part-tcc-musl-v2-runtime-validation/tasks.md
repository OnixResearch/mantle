# Tasks: Complete tcc musl v2 runtime validation

## Validation

- [x] V1 Confirm prerequisite runtime blockers (make 3.82, tcc-musl, rebuilt musl) are resolved or record current blockers. Evidence: `evidence/V1-prerequisite-blockers.md`. [covers=bootstrap.part.tcc.musl.v2.runtime-validation]
- [x] V2 Build `bootstrap/tcc-musl-v2.ncl` and record output path or failure class plus provider/fallback/placeholder fields. Evidence: `evidence/V2-build.md` and `evidence/V2-tcc-musl-v2-pass-*`. [covers=bootstrap.part.tcc.musl.v2.runtime-validation]
- [x] V3 Smoke-test installed `tcc`, `tcc-0.9.27-musl-v2`, `libtcc1.a`, and trivial C compilation. Evidence: `evidence/V3-smoke.md`. [covers=bootstrap.part.tcc.musl.v2.runtime-validation]
- [x] V4 Scan derivation and transcript for undeclared host-tool/path/environment leakage. Evidence: `evidence/V4-host-leakage.md`. [covers=bootstrap.part.tcc.musl.v2.runtime-validation]
- [x] V5 Run OpenSpec validation/gates before archive. Evidence: `evidence/V5-openspec-validate.md`. [covers=bootstrap.part.tcc.musl.v2.runtime-validation]
