# Tasks: Complete gcc-4.7 runtime validation

## Validation

- [ ] V1 Confirm binutils-tcc and gcc-4.0 runtime validation evidence is available or record blockers. [covers=bootstrap.gcc47.runtime-validation]
- [ ] V2 Build `bootstrap/gcc-4.7.ncl` and record required transcript fields. [covers=bootstrap.gcc47.runtime-validation]
- [ ] V3 Validate no host compiler/libc/shell/Nix/legacy-provider leakage. [covers=bootstrap.gcc47.runtime-validation]
- [ ] V4 Run C, C++, and minimal C++11 compiler smoke tests. [covers=bootstrap.gcc47.runtime-validation]
- [ ] V5 Run OpenSpec validation/gates before archive. [covers=bootstrap.gcc47.runtime-validation]
