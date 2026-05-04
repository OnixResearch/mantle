# Tasks: Complete tcc musl runtime validation

## Validation

- [x] V1 Confirm prerequisite runtime blockers (make 3.82, first musl pass, tcc-musl-prep) are resolved or record current blockers. Evidence: `evidence/V1-prerequisite-blockers.md` (`tcc-0.9.27-musl-prep.drv` blocks before this target builder). [covers=bootstrap.part.tcc.musl.runtime-validation]
- [x] V2 Build `bootstrap/tcc-musl.ncl` and record output path or failure class plus provider/fallback/placeholder fields. Evidence: `evidence/V2-build.md` (build failed before output at prerequisite `tcc-0.9.27-musl-prep.drv`). [covers=bootstrap.part.tcc.musl.runtime-validation]
- [ ] V3 Smoke-test installed `tcc`, `tcc-0.9.27-musl`, `libtcc1.a`, and trivial C compilation. Blocked until `tcc-0.9.27-musl-prep` produces a usable prerequisite output; see `evidence/V3-smoke.md`. [covers=bootstrap.part.tcc.musl.runtime-validation]
- [x] V4 Scan derivation and transcript for undeclared host-tool/path/environment leakage. Evidence: `evidence/V4-host-leakage.md` (failed prerequisite attempt only; rerun after successful build). [covers=bootstrap.part.tcc.musl.runtime-validation]
- [ ] V5 Run OpenSpec validation/gates before archive. [covers=bootstrap.part.tcc.musl.runtime-validation]
