# Tasks: Complete patch 2.5.9 runtime validation

## Validation

- [x] V1 Confirm prerequisite runtime blockers (make 3.82 and TinyCC amd64 repair) are resolved or record current blockers. Evidence: `evidence/V1-prerequisites.md`. [covers=bootstrap.part.patch.2.5.9.runtime-validation]
- [x] V2 Build `bootstrap/patch-tcc.ncl` and record output path or failure class plus provider/fallback/placeholder fields. Evidence: `evidence/V2-V4-runtime-validation.md`, `evidence/build.stdout.log`, `evidence/validation-summary.json`. [covers=bootstrap.part.patch.2.5.9.runtime-validation]
- [x] V3 Smoke-test produced `patch` with `--version` and simple unified-diff application. Evidence: `evidence/patch-tcc.drv.log`. [covers=bootstrap.part.patch.2.5.9.runtime-validation]
- [x] V4 Scan derivation and transcript for undeclared host-tool/path/environment leakage. Evidence: `evidence/V2-V4-runtime-validation.md`, `evidence/validation-summary.md`. [covers=bootstrap.part.patch.2.5.9.runtime-validation]
- [x] V5 Run OpenSpec validation/gates before archive. Evidence: `openspec validate live-part-patch-2-5-9-runtime-validation --strict --json`, `openspec_helper.py verify live-part-patch-2-5-9-runtime-validation --json` (clean except in-progress V5 before this final mark), `git diff --check`. [covers=bootstrap.part.patch.2.5.9.runtime-validation]
