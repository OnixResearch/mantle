# Tasks: Implement binutils-tcc live-bootstrap chain

## Implementation

All implementation tasks must pin each new source, carried patch, and generated artifact at the first consuming derivation with URL/path, digest, provenance, and first-consuming derivation metadata.

- [ ] I1 Implement early tcc-hosted utility derivations: `bzip2-1.0.8`, `coreutils-5.0`, `oyacc-6.6`, and `bash-2.05b`. [covers=bootstrap.binutils.tcc.chain]
- [ ] I2 Implement first libc/compiler boundary derivations: patched `tcc-0.9.27`, `musl-1.1.24`, `tcc-0.9.27-musl`, rebuilt `musl-1.1.24`, and `tcc-0.9.27-musl-v2`. [covers=bootstrap.binutils.tcc.chain]
- [ ] I3 Implement post-musl text/parser tools: `grep-2.4`, rebuilt `sed-4.0.9`, rebuilt `bzip2-1.0.8`, `m4-1.4.7`, `heirloom-devtools-070527`, `flex-2.5.11`, `flex-2.6.4`, `bison-2.3`, and `bison-3.4.1`. [covers=bootstrap.binutils.tcc.chain]
- [ ] I4 Implement late utility and autotools ladder: `diffutils-2.7`, rebuilt `coreutils-5.0`, `coreutils-6.10`, `gawk-3.0.4`, `perl-5.000`, `perl-5.003`, `perl-5.004_05`, `perl-5.005_03`, `perl-5.6.2`, `autoconf-2.52`, `automake-1.6.3`, `autoconf-2.53`, `automake-1.7`, `autoconf-2.54`, `autoconf-2.55`, `automake-1.7.8`, `autoconf-2.57`, `autoconf-2.59`, `automake-1.8.5`, `autoconf-2.61`, `automake-1.9.6`, `automake-1.10.3`, `autoconf-2.64`, `automake-1.11.2`, `autoconf-2.69`, `libtool-2.2.4`, and `automake-1.15.1`. [covers=bootstrap.binutils.tcc.chain]
- [ ] I5 Replace `bootstrap/binutils-tcc.ncl` placeholder with functional binutils 2.30 output installing `as`, `ld`, `ar`, `ranlib`, `nm`, and `objcopy`. [covers=bootstrap.binutils.tcc.chain]

## Validation

All validation evidence must include command, provider selection, exit status, output path or failure class, fallback status/event marker, and placeholder rejection result.

- [ ] V1 Run source-pin audit for all new sources, patches, and generated artifacts. [covers=bootstrap.binutils.tcc.chain] [evidence=evidence/V1-source-pins.md]
- [ ] V2 Build every epoch derivation in dependency order with `crunch build <file>` and record the required transcript fields. [covers=bootstrap.binutils.tcc.chain] [evidence=evidence/V2-epoch-builds.md]
- [ ] V3 Validate no host compiler/libc/shell/Nix/legacy-provider leakage in intermediate transcripts. [covers=bootstrap.binutils.tcc.chain] [evidence=evidence/V3-host-leakage.md]
- [ ] V4 Validate post-musl `m4`, `flex`, `bison`, and `grep` link against musl. [covers=bootstrap.binutils.tcc.chain] [evidence=evidence/V4-musl-linkage.md]
- [ ] V5 Validate binutils 2.30 tools and assembler smoke test. [covers=bootstrap.binutils.tcc.chain] [evidence=evidence/V5-binutils-smoke.md]
- [ ] V6 Run OpenSpec validation and gates before archive. [covers=bootstrap.binutils.tcc.chain] [evidence=evidence/V6-openspec-gates.md]
