# Tasks: Implement gcc-4.0.4 bootstrap stage

## Implementation

- [ ] I1 Replace `bootstrap/gcc-4.0.ncl` placeholder with gcc 4.0.4 C and C++ compiler outputs using chain-internal `binutils-tcc` inputs. [covers=bootstrap.gcc40.transition]
- [ ] I2 Pin gcc 4.0.4 C/C++ sources and carried/generated artifacts with URL/path, digest, provenance, and first-consuming derivation metadata. [covers=bootstrap.gcc40.transition]
- [ ] I3 Add `./scripts/check-bootstrap-transcript.rs` with `--reject-host-tools` support if no existing transcript checker covers host compiler/libc/shell/Nix/legacy-provider fallback rejection. [covers=bootstrap.gcc40.transition]

## Validation

- [ ] V1 Run source-pin audit for gcc-4.0.4 artifacts. [covers=bootstrap.gcc40.transition] [evidence=evidence/V1-source-pins.md]
- [ ] V2 Build `bootstrap/gcc-4.0.ncl` and record transcript fields. [covers=bootstrap.gcc40.transition] [evidence=evidence/V2-build.md]
- [ ] V3 Validate no host compiler/libc/shell/Nix/legacy-provider leakage with `./scripts/check-bootstrap-transcript.rs --reject-host-tools openspec/changes/live-bootstrap-gcc-4-0-stage/evidence/V2-build.md`. [covers=bootstrap.gcc40.transition] [evidence=evidence/V3-host-leakage.md]
- [ ] V4 Run C and C++ compiler smoke tests with the gcc-4.0.4 output. [covers=bootstrap.gcc40.transition] [evidence=evidence/V4-compiler-smoke.md]
- [ ] V5 Run OpenSpec validation and gates before archive. [covers=bootstrap.gcc40.transition] [evidence=evidence/V5-openspec-gates.md]
