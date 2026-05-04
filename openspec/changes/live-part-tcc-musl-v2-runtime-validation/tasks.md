# Tasks: Complete tcc musl v2 runtime validation

## Validation

- [x] V1 Confirm prerequisite runtime blockers (make 3.82, tcc-musl, rebuilt musl) are resolved or record current blockers. Evidence: `evidence/V1-prerequisite-blockers.md` (`sed-4.0.9-tcc.drv` blocks before this stage reaches the listed later prerequisites). [covers=bootstrap.part.tcc.musl.v2.runtime-validation]
- [x] V2 Build `bootstrap/tcc-musl-v2.ncl` and record output path or failure class plus provider/fallback/placeholder fields. Evidence: `evidence/V2-build.md` (build failed before output at `sed-4.0.9-tcc.drv`). [covers=bootstrap.part.tcc.musl.v2.runtime-validation]
- [ ] V3 Smoke-test installed `tcc`, `tcc-0.9.27-musl-v2`, `libtcc1.a`, and trivial C compilation. Blocked until `sed-4.0.9-tcc` produces a usable prerequisite output; see `evidence/V3-smoke.md`. [covers=bootstrap.part.tcc.musl.v2.runtime-validation]
- [x] V4 Scan derivation and transcript for undeclared host-tool/path/environment leakage. Evidence: `evidence/V4-host-leakage.md` (failed prerequisite attempt only; rerun after successful build). [covers=bootstrap.part.tcc.musl.v2.runtime-validation]
- [ ] V5 Run OpenSpec validation/gates before archive. [covers=bootstrap.part.tcc.musl.v2.runtime-validation]
