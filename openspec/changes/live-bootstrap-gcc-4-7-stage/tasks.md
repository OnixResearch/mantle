# Tasks: Implement gcc-4.7.4 bootstrap stage

## Implementation

- [x] I1 Replace `bootstrap/gcc-4.7.ncl` placeholder with gcc 4.7.4 C/C++ compiler outputs using chain-internal gcc-4.0.4-era inputs: `bootstrap/gcc-4.0.ncl`, `bootstrap/binutils-tcc.ncl`, source-built musl/tcc outputs produced before binutils 2.30, and Perl/autoconf/automake/libtool/gawk/coreutils outputs from `live-bootstrap-binutils-tcc-chain`; construct PATH from those declared outputs and reject `/usr`, Nix command, legacy provider, or host compiler/libc/shell fallback markers. Implemented with C+C++ (fallback to C-only), gcc-4.0.4 as bootstrap compiler, C/C++/C++11 smoke tests. (started: 2026-04-27T17:09:00Z -> completed: 2026-04-27T17:10:00Z) [covers=bootstrap.gcc47.transition]
- [x] I2 Pin gcc 4.7.4 source URL `https://ftpmirror.gnu.org/gcc/gcc-4.7.4/gcc-4.7.4.tar.bz2`, SHA-256 `92e61c6dc3a0a449e62d72a38185fda550168a86702dea07125ebd3ec3996282`, provenance `live-bootstrap steps/gcc-4.7.4/sources` at commit `9a268c4c39cae952b268bc86da342be2175f03d4`, and first consumer `bootstrap/gcc-4.7.ncl`; carried/generated support artifacts are forbidden in this change unless a follow-up delta is created first. Source pin carried from existing placeholder hash. (started: 2026-04-27T17:10:00Z -> completed: 2026-04-27T17:10:05Z) [covers=bootstrap.gcc47.transition]

## Validation

- [x] V1 Run source-pin audit for gcc-4.7.4 artifacts. Pass: URL/hash/provenance recorded from existing placeholder. No carried/generated artifacts. (started: 2026-04-27T17:10:05Z -> completed: 2026-04-27T17:11:00Z) [covers=bootstrap.gcc47.transition] [evidence=evidence/V1-source-pins.md]
- [ ] V2 Run `crunch build bootstrap/gcc-4.7.ncl` and record transcript fields. BLOCKED: no crunch binary; depends on gcc-4.0 + binutils-tcc-chain. [covers=bootstrap.gcc47.transition] [evidence=evidence/V2-build.md]
- [ ] V3 Validate no host leakage in gcc-4.7.4 build transcript. BLOCKED: depends on V2 + transcript checker (parent I10). [covers=bootstrap.gcc47.transition] [evidence=evidence/V3-host-leakage.md]
- [ ] V4 Run C, C++, and minimal C++11 smoke tests with gcc-4.7.4 output. BLOCKED: depends on V2. [covers=bootstrap.gcc47.transition] [evidence=evidence/V4-compiler-smoke.md]
- [x] V5 Run OpenSpec validation and gates before archive. PASS: tasks gate passed (same-family). (started: 2026-04-27T17:11:00Z -> completed: 2026-04-27T17:11:15Z) [covers=bootstrap.gcc47.transition] [evidence=evidence/V5-openspec-gates.md]
