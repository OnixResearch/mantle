# Tasks: Complete tcc musl prep runtime validation

## Validation

- [x] V1 Confirm prerequisite runtime blockers (make 3.82 and TinyCC amd64 repair) are resolved or record current blockers. Evidence: `evidence/V1-prerequisite-blockers.md` (validation reached the `tcc-0.9.27-musl-prep` builder). [covers=bootstrap.part.tcc.musl.prep.runtime-validation]
- [x] V2 Build `bootstrap/tcc-musl-prep.ncl` and record output path or failure class plus provider/fallback/placeholder fields. Evidence: `evidence/V2-build.md` (build failed in `elf.h` parsing: `Elf32_Xword`). [covers=bootstrap.part.tcc.musl.prep.runtime-validation]
- [ ] V3 Smoke-test installed `tcc`, `tcc-musl-prep`, carried Mes libc/headers, and `tcc -v`. Blocked until V2 produces an output; see `evidence/V3-smoke.md`. [covers=bootstrap.part.tcc.musl.prep.runtime-validation]
- [x] V4 Scan derivation and transcript for undeclared host-tool/path/environment leakage. Evidence: `evidence/V4-host-leakage.md` (failed attempt only; rerun after successful build). [covers=bootstrap.part.tcc.musl.prep.runtime-validation]
- [ ] V5 Run OpenSpec validation/gates before archive. [covers=bootstrap.part.tcc.musl.prep.runtime-validation]
