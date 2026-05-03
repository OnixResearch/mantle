# Tasks: Complete make 3.82 runtime validation

## Validation

- [x] V1 Confirm `repair-make-tcc-amd64-varargs` is complete or record its current blocker. [covers=bootstrap.part.make.3.82.runtime-validation] Evidence: `evidence/V1-prerequisite-status.md`.
- [x] V2 Build `bootstrap/make-tcc.ncl` and record output path or failure class plus provider/fallback/placeholder fields. [covers=bootstrap.part.make.3.82.runtime-validation] Evidence: `evidence/V2-build-attempt.md` (`incomplete-hung-validation-run`), `evidence/V2-build-rerun.md` (`build`, exit 139), and `evidence/V2-build-rerun-2026-05-02.md` (reproduced `make-3.82-tcc` exit 139).
- [ ] V3 Smoke-test `make --version` and simple Makefile execution with the produced GNU Make 3.82 output. [covers=bootstrap.part.make.3.82.runtime-validation] Blocked evidence: `evidence/V3-smoke-blocked.md` (updated with 2026-05-02 rerun), `evidence/diag-tcc27-static-runtime.md` (2026-05-03 smaller TinyCC/Mes static executable also segfaults at runtime), and `evidence/diag-tcc27-static-runtime-inspect.md` (link-only seam diagnostic added; foreground rerun timed out before reaching diagnostic builder output).
- [x] V4 Scan derivation and transcript for undeclared host-tool/path/environment leakage. [covers=bootstrap.part.make.3.82.runtime-validation] Evidence: `evidence/V4-host-leakage.md` and `evidence/V4-host-leakage-2026-05-02.md`.
- [ ] V5 Run OpenSpec validation/gates before archive. [covers=bootstrap.part.make.3.82.runtime-validation] Current evidence: `evidence/V5-openspec-validate-current.json` (CLI validation passes), `evidence/V5-openspec-verify-current.json`, and `evidence/V5-openspec-verify-2026-05-02.log` (expected helper warning: V3/V5 remain incomplete).
