# Tasks: Complete gcc-4.0 runtime validation

## Validation

- [ ] V1 Confirm binutils-tcc runtime validation evidence is available or record its blocker. [covers=bootstrap.gcc40.runtime-validation]
- [ ] V2 Build `bootstrap/gcc-4.0.ncl` and record required transcript fields. [covers=bootstrap.gcc40.runtime-validation]
- [ ] V3 Validate no host compiler/libc/shell/Nix/legacy-provider leakage. [covers=bootstrap.gcc40.runtime-validation]
- [ ] V4 Run C and C++ compiler smoke tests with the gcc-4.0.4 output. [covers=bootstrap.gcc40.runtime-validation]
- [ ] V5 Run OpenSpec validation/gates before archive. [covers=bootstrap.gcc40.runtime-validation]
