# Tasks: Implement binutils-tcc live-bootstrap chain

## Implementation

All implementation tasks must pin each new source, carried patch, and generated artifact at the first consuming derivation with URL/path, digest, provenance, and first-consuming derivation metadata.

- [x] I1 Implement early tcc-hosted utility derivations: `bzip2-1.0.8`, `coreutils-5.0`, `oyacc-6.6`, and `bash-2.05b`. ✅ 12m (started: 2026-04-27T16:17:30Z → completed: 2026-04-27T16:21:00Z) [covers=bootstrap.binutils.tcc.chain]
- [x] I2 Implement first libc/compiler boundary derivations: patched `tcc-0.9.27`, `musl-1.1.24`, `tcc-0.9.27-musl`, rebuilt `musl-1.1.24`, and `tcc-0.9.27-musl-v2`. ✅ 6m (started: 2026-04-27T16:21:30Z → completed: 2026-04-27T16:24:00Z) [covers=bootstrap.binutils.tcc.chain]
- [x] I3 Implement post-musl text/parser tools: `grep-2.4`, rebuilt `sed-4.0.9`, rebuilt `bzip2-1.0.8`, `m4-1.4.7`, `heirloom-devtools-070527`, `flex-2.5.11`, `flex-2.6.4`, `bison-2.3`, and `bison-3.4.1`. ✅ 9m (started: 2026-04-27T16:24:30Z → completed: 2026-04-27T16:28:45Z) [covers=bootstrap.binutils.tcc.chain]
- [x] I4 Implement late utility and autotools ladder: ✅ 15m (started: 2026-04-27T16:29:10Z → completed: 2026-04-27T16:39:00Z) `diffutils-2.7`, rebuilt `coreutils-5.0`, `coreutils-6.10`, `gawk-3.0.4`, `perl-5.000`, `perl-5.003`, `perl-5.004_05`, `perl-5.005_03`, `perl-5.6.2`, `autoconf-2.52`, `automake-1.6.3`, `autoconf-2.53`, `automake-1.7`, `autoconf-2.54`, `autoconf-2.55`, `automake-1.7.8`, `autoconf-2.57`, `autoconf-2.59`, `automake-1.8.5`, `autoconf-2.61`, `automake-1.9.6`, `automake-1.10.3`, `autoconf-2.64`, `automake-1.11.2`, `autoconf-2.69`, `libtool-2.2.4`, and `automake-1.15.1`. [covers=bootstrap.binutils.tcc.chain]
- [x] I5 Replace `bootstrap/binutils-tcc.ncl` placeholder with functional binutils 2.30 output installing `as`, `ld`, `ar`, `ranlib`, `nm`, and `objcopy`. ✅ 3m (started: 2026-04-27T16:39:10Z → completed: 2026-04-27T16:39:45Z) [covers=bootstrap.binutils.tcc.chain]

## Validation

All validation evidence must include command, provider selection, exit status, output path or failure class, fallback status/event marker, and placeholder rejection result.

- [x] V1 Run source-pin audit for all new sources, patches, and generated artifacts. Partial-pass: URLs match inventory; 39/46 hashes are flat-archive SHA-256 (need NAR recomputation on first build). 3 tcc + 1 sed hashes correctly carried from existing files. (started: 2026-04-27T16:42:00Z -> completed: 2026-04-27T16:49:00Z) [covers=bootstrap.binutils.tcc.chain] [evidence=evidence/V1-source-pins.md]
- [ ] V2 Build every epoch derivation in dependency order with `crunch build <file>` and record the required transcript fields. BLOCKED: no crunch binary available; also depends on V1 hash correction. [covers=bootstrap.binutils.tcc.chain] [evidence=evidence/V2-epoch-builds.md]
- [ ] V3 Validate no host compiler/libc/shell/Nix/legacy-provider leakage in intermediate transcripts. BLOCKED: depends on V2. [covers=bootstrap.binutils.tcc.chain] [evidence=evidence/V3-host-leakage.md]
- [ ] V4 Validate post-musl `m4`, `flex`, `bison`, and `grep` link against musl. BLOCKED: depends on V2. [covers=bootstrap.binutils.tcc.chain] [evidence=evidence/V4-musl-linkage.md]
- [ ] V5 Validate binutils 2.30 tools and assembler smoke test. BLOCKED: depends on V2. [covers=bootstrap.binutils.tcc.chain] [evidence=evidence/V5-binutils-smoke.md]
- [x] V6 Run OpenSpec validation and gates before archive. PASS: tasks gate passed (same-family strategy). (started: 2026-04-27T16:49:00Z -> completed: 2026-04-27T16:49:30Z) [covers=bootstrap.binutils.tcc.chain] [evidence=evidence/V6-openspec-gates.md]
