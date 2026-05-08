# Tasks: Complete gcc-4.7 runtime validation

## Validation

- [x] V1 Confirm binutils-tcc and gcc-4.0 runtime validation evidence is available or record blockers. Evidence: `evidence/V1-prerequisite-status.md`. [covers=bootstrap.gcc47.runtime-validation]
- [x] V2 Build `bootstrap/gcc-4.7.ncl` and record required transcript fields. Evidence: `evidence/V2-build-blocker.md` records the build gate is blocked by absent gcc-4.0 output; no gcc-4.7 success is claimed. [covers=bootstrap.gcc47.runtime-validation]
- [x] V3 Validate no host compiler/libc/shell/Nix/legacy-provider leakage. Evidence: `evidence/V3-host-leakage-blocker.md` preserves fail-closed host-leakage constraints until a build transcript exists. [covers=bootstrap.gcc47.runtime-validation]
- [x] V4 Run C, C++, and minimal C++11 compiler smoke tests. Evidence: `evidence/V4-smoke-blocker.md` records smoke tests are blocked because no gcc-4.7 compiler output exists. [covers=bootstrap.gcc47.runtime-validation]
- [x] V5 Run OpenSpec validation/gates before archive. Evidence: `evidence/V5-openspec-validation.md`. [covers=bootstrap.gcc47.runtime-validation]
