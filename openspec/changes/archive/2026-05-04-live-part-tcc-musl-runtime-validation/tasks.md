# Tasks: Complete tcc musl runtime validation

## Validation

- [x] V1 Confirm prerequisite runtime blockers (make 3.82, first musl pass, tcc-musl-prep) are resolved or record current blockers. Evidence: `evidence/V1-prerequisite-blockers.md` (`musl-1.1.24-tcc` now passes focused validation; current blocker has advanced to `tcc-0.9.27-musl.drv`). [covers=bootstrap.part.tcc.musl.runtime-validation]
- [x] V2 Build `bootstrap/tcc-musl.ncl` and record output path or failure class plus provider/fallback/placeholder fields. Evidence: `evidence/V2-build.md` (build still fails before output, now inside `tcc-0.9.27-musl.drv` after first musl passes). [covers=bootstrap.part.tcc.musl.runtime-validation]
- [x] V3 Smoke-test installed `tcc`, `tcc-0.9.27-musl`, `libtcc1.a`, and trivial C compilation. Evidence: `evidence/V3-smoke.md` and `evidence/V2-tcc-musl-link-repair-*`. [covers=bootstrap.part.tcc.musl.runtime-validation]
- [x] V4 Scan derivation and transcript for undeclared host-tool/path/environment leakage. Evidence: `evidence/V4-host-leakage.md` (no leakage findings in the passing first-musl validation or downstream tcc-musl failure summary). [covers=bootstrap.part.tcc.musl.runtime-validation]
- [x] V5 Run OpenSpec validation/gates before archive. Evidence: `evidence/V5-openspec-validate.md`. [covers=bootstrap.part.tcc.musl.runtime-validation]
