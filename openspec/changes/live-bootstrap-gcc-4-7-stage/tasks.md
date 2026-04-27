# Tasks: Implement gcc-4.7.4 bootstrap stage

## Implementation

- [ ] I1 Replace `bootstrap/gcc-4.7.ncl` placeholder with gcc 4.7.4 C/C++ compiler outputs using chain-internal gcc-4.0.4-era inputs: `bootstrap/gcc-4.0.ncl`, `bootstrap/binutils-tcc.ncl`, source-built musl/tcc outputs produced before binutils 2.30, and Perl/autoconf/automake/libtool/gawk/coreutils outputs from `live-bootstrap-binutils-tcc-chain`; construct PATH from those declared outputs and reject `/usr`, Nix command, legacy provider, or host compiler/libc/shell fallback markers. [covers=bootstrap.gcc47.transition]
- [ ] I2 Pin gcc 4.7.4 source URL `https://ftpmirror.gnu.org/gcc/gcc-4.7.4/gcc-4.7.4.tar.bz2`, SHA-256 `92e61c6dc3a0a449e62d72a38185fda550168a86702dea07125ebd3ec3996282`, provenance `live-bootstrap steps/gcc-4.7.4/sources` at commit `9a268c4c39cae952b268bc86da342be2175f03d4`, and first consumer `bootstrap/gcc-4.7.ncl`; carried/generated support artifacts are forbidden in this change unless a follow-up delta is created first. [covers=bootstrap.gcc47.transition]

## Validation

- [ ] V1 Run `./scripts/check-bootstrap-source-pins.rs bootstrap/gcc-4.7.ncl` and prove it reports URL `https://ftpmirror.gnu.org/gcc/gcc-4.7.4/gcc-4.7.4.tar.bz2`, SHA-256 `92e61c6dc3a0a449e62d72a38185fda550168a86702dea07125ebd3ec3996282`, provenance commit `9a268c4c39cae952b268bc86da342be2175f03d4`, first consumer `bootstrap/gcc-4.7.ncl`, and no carried/generated support artifact consumed without a follow-up delta. [covers=bootstrap.gcc47.transition] [evidence=evidence/V1-source-pins.md]
- [ ] V2 Run `crunch build bootstrap/gcc-4.7.ncl` and record transcript fields: command, provider selection, exit status, output path or failure class, fallback status/event marker, and placeholder rejection result. [covers=bootstrap.gcc47.transition] [evidence=evidence/V2-build.md]
- [ ] V3 Run `./scripts/check-bootstrap-transcript.rs --reject-host-tools evidence/V2-build.md` to validate declared-chain PATH construction, `fallback-event=none`, rejection of any `fallback-event=<kind>` other than `none`, and no host compiler/libc/shell/Nix/legacy-provider leakage. [covers=bootstrap.gcc47.transition] [evidence=evidence/V3-host-leakage.md]
- [ ] V4 Run C, C++, and minimal C++11 smoke tests with gcc-4.7.4 output, including a named C++11 source that exercises a C++11-only construct. [covers=bootstrap.gcc47.transition] [evidence=evidence/V4-compiler-smoke.md]
- [ ] V5 Run OpenSpec validation and gates before archive. [covers=bootstrap.gcc47.transition] [evidence=evidence/V5-openspec-gates.md]
