# Tasks: Complete tcc musl prep runtime validation

## Validation

- [x] V1 Confirm prerequisite runtime blockers (make 3.82 and TinyCC amd64 repair) are resolved or record current blockers. Evidence: `evidence/V1-prerequisite-blockers.md` (validation reached and passed the `tcc-0.9.27-musl-prep` builder). [covers=bootstrap.part.tcc.musl.prep.runtime-validation]
- [x] V2 Build `bootstrap/tcc-musl-prep.ncl` and record output path or failure class plus provider/fallback/placeholder fields. Evidence: `evidence/V2-build.md` (build passed and exported the bridge compiler output). [covers=bootstrap.part.tcc.musl.prep.runtime-validation]
- [x] V3 Smoke-test installed `tcc`, `tcc-musl-prep`, carried Mes libc/headers, and `tcc -v`. Evidence: `evidence/V3-smoke.md`. [covers=bootstrap.part.tcc.musl.prep.runtime-validation]
- [x] V4 Scan derivation and transcript for undeclared host-tool/path/environment leakage. Evidence: `evidence/V4-host-leakage.md` (no coarse host-path needles found). [covers=bootstrap.part.tcc.musl.prep.runtime-validation]
- [x] V5 Run OpenSpec validation/gates before archive. Evidence: `evidence/V5-openspec-verify.json` and final clean helper rerun. [covers=bootstrap.part.tcc.musl.prep.runtime-validation]
