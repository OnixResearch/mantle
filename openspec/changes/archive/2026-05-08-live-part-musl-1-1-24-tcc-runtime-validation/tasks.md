# Tasks: Complete musl 1.1.24 tcc runtime validation

## Validation

- [x] V1 Confirm prerequisite runtime blockers (`repair-make-tcc-amd64-varargs` and make 3.82 runtime validation) are resolved or record current blockers. Evidence: `evidence/V1-prerequisite-status.md`. [covers=bootstrap.part.musl.1.1.24.tcc.runtime-validation]
- [x] V2 Build `bootstrap/musl-1.1.24-tcc.ncl` and record output path or failure class plus provider/fallback/placeholder fields. Evidence: `evidence/V2-validation-run.md`, `evidence/doctor.json`, and checkpoint `evidence/validation-summary.json`; failure class `incomplete-hung-mes-prerequisite`, no musl output path. [covers=bootstrap.part.musl.1.1.24.tcc.runtime-validation]
- [x] V3 Smoke-test produced `libc.a`, headers, and startup object contract. Evidence: `evidence/V3-output-contract-blocked.md` records smoke is blocked because no musl output exists; no contract success is claimed. [covers=bootstrap.part.musl.1.1.24.tcc.runtime-validation]
- [x] V4 Scan derivation and transcript for undeclared host-tool/path/environment leakage. Evidence: `evidence/V4-host-leakage-status.md` preserves fail-closed host-leakage constraints and notes no completed musl transcript exists. [covers=bootstrap.part.musl.1.1.24.tcc.runtime-validation]
- [x] V5 Run OpenSpec validation/gates before archive. Evidence: `evidence/V5-openspec-validation.md`; strict OpenSpec validation and `git diff --check` passed before archive. [covers=bootstrap.part.musl.1.1.24.tcc.runtime-validation]
