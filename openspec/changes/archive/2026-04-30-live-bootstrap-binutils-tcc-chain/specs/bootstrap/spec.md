## ADDED Requirements

### Requirement: Binutils-TCC chain implementation

Crunch MUST build `bootstrap/binutils-tcc.ncl` from chain-internal TinyCC-era and post-musl derivations without using host compiler, host libc, host shell tools, or the legacy musl.cc provider.
ID: bootstrap.binutils.tcc.chain

The chain MUST implement the scoped ladder groups named by the proposal: early tcc-hosted utilities (`bzip2`, `coreutils-5.0`, `oyacc`, `bash-2.05b`), first musl/tcc rebuilds, post-musl text/parser tools (`grep`, rebuilt `sed`, rebuilt `bzip2`, `m4`, Heirloom devtools, `flex`, `bison`), diffutils/coreutils/gawk, Perl/autoconf/automake/libtool, and binutils 2.30. The chain MUST pin every source, carried patch, and generated artifact with URL or repository path, digest, and provenance at the first consuming derivation. Validation transcripts MUST record command, provider selection, exit status, output path, fallback status/event marker, and placeholder rejection result. Validation MUST prove post-musl `m4`, `flex`, `bison`, and `grep` link against musl, and MUST prove binutils 2.30 can assemble a trivial ELF object for the gcc-4.0.4 transition.

#### Scenario: Placeholder is replaced

- GIVEN `bootstrap/binutils-tcc.ncl` is evaluated
- WHEN the derivation builds
- THEN it does not emit `ERROR: binutils-tcc.ncl is a placeholder`
- AND it produces working `as`, `ld`, `ar`, `ranlib`, `nm`, and `objcopy` tools

#### Scenario: Host leakage is rejected

- GIVEN any intermediate tool derivation invokes a host compiler, host libc, host shell tool, Nix command, or legacy provider executable
- WHEN validation audits the transcript
- THEN the chain is marked incomplete
- AND full-source bootstrap status remains blocked

#### Scenario: Source provenance is complete

- GIVEN a new source, patch, or generated artifact is consumed by the binutils-TCC chain
- WHEN the source-pin audit runs
- THEN it records URL or repository path, digest, provenance, and first consuming derivation
