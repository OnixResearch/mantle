## ADDED Requirements

### Requirement: GCC 4.0.4 transition stage

Crunch MUST build `bootstrap/gcc-4.0.ncl` as gcc 4.0.4 C and C++ compiler outputs using only the chain-internal TinyCC/musl/binutils 2.30 inputs.
ID: bootstrap.gcc40.transition

The stage MUST pin gcc 4.0.4 C/C++ source inputs and every carried patch or generated artifact with URL/path, digest, provenance, and first-consuming derivation metadata. Validation MUST prove the output compiles C and C++ smoke programs and MUST reject host compiler, host libc, host shell, Nix, or legacy-provider fallback by scanning sandbox command transcripts and proof markers.

#### Scenario: GCC 4.0.4 placeholder is replaced

- GIVEN `bootstrap/gcc-4.0.ncl` is built
- WHEN the build completes
- THEN it does not emit `ERROR: gcc-4.0.ncl is a placeholder`
- AND it produces working C and C++ compiler binaries

#### Scenario: GCC 4.0.4 rejects host leakage

- GIVEN the stage transcript contains host compiler, host libc, host shell, Nix, or legacy provider execution
- WHEN validation runs
- THEN the stage is marked incomplete
- AND parent full-source bootstrap status remains blocked
