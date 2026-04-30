## ADDED Requirements

### Requirement: GCC 4.7.4 transition stage

Crunch MUST build `bootstrap/gcc-4.7.ncl` as gcc 4.7.4 C and C++ compiler outputs using only chain-internal gcc-4.0.4-era inputs.
ID: bootstrap.gcc47.transition

The stage MUST pin gcc 4.7.4 and support artifacts with URL/path, digest, provenance, and first-consuming derivation metadata. Validation MUST prove the output compiles C, C++, and minimal C++11 smoke programs, and MUST reject host compiler, host libc, host shell, Nix, or legacy-provider fallback by scanning sandbox command transcripts and proof markers.

#### Scenario: GCC 4.7.4 placeholder is replaced

- GIVEN `bootstrap/gcc-4.7.ncl` is built
- WHEN the build completes
- THEN it does not emit `ERROR: gcc-4.7.ncl is a placeholder`
- AND it produces working C and C++ compiler binaries

#### Scenario: C++11 transition is usable

- GIVEN gcc 4.7.4 output exists
- WHEN validation compiles a minimal C++11 source
- THEN the compile succeeds using only chain-internal inputs
