# Tasks: Complete gcc-4.0 runtime validation

## Validation

- [x] V1 Confirm binutils-tcc runtime validation evidence is available or record its blocker. Evidence: `evidence/V1-binutils-prereq.md`. [covers=bootstrap.gcc40.runtime-validation]
- [x] V2 Build `bootstrap/gcc-4.0.ncl` and record required transcript fields. Evidence: `evidence/V2-gcc40-libiberty-cpp-boundary.md`; follow-up boundaries: `evidence/V2-gcc40-libiberty-cplus-dem-boundary.md`, `evidence/V2-gcc40-libcpp-charset-boundary.md`, `evidence/V2-gcc40-gcc-genmodes-boundary.md`, `evidence/V2-gcc40-gcc-genmddeps-boundary.md`. [covers=bootstrap.gcc40.runtime-validation]
- [ ] V3 Validate no host compiler/libc/shell/Nix/legacy-provider leakage. Current validation reports only coarse `/bin/` matches from captured configure output; keep open until a passing or cleaner boundary is recorded. [covers=bootstrap.gcc40.runtime-validation]
- [ ] V4 Run C and C++ compiler smoke tests with the gcc-4.0.4 output. [covers=bootstrap.gcc40.runtime-validation]
- [ ] V5 Run OpenSpec validation/gates before archive. [covers=bootstrap.gcc40.runtime-validation]
