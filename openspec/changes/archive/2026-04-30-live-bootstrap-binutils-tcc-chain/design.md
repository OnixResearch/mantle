# Design: Binutils-TCC live-bootstrap chain

## Context

`bootstrap/binutils-tcc.ncl` currently documents the intermediate chain and exits
as a placeholder. The parent inventory shows this is not one derivation but a
ladder from early tcc-hosted tools through musl/tcc rebuilds, post-musl parser
and autotools tools, Perl/autoconf/automake/libtool, then binutils 2.30.

## Decisions

### 1. Split implementation by exact chain epochs

**Choice:** implement these derivation epochs in order, preserving exact versions
from the parent I1 inventory and `fosslinux/live-bootstrap` commit
`9a268c4c39cae952b268bc86da342be2175f03d4`:

1. Existing base: `stage0-posix`, `mes`, `tinycc`, `make-tcc`, `sed-tcc`,
   `patch-tcc`, `gzip-tcc`, `tar-tcc`.
2. Early tcc-hosted utilities: `bzip2-1.0.8`, `coreutils-5.0`, `oyacc-6.6`,
   `bash-2.05b`.
3. First libc/compiler boundary: patched `tcc-0.9.27`, `musl-1.1.24`,
   `tcc-0.9.27-musl`, rebuilt `musl-1.1.24`, and `tcc-0.9.27-musl-v2`.
4. Post-musl parser/text tools: `grep-2.4`, rebuilt `sed-4.0.9`, rebuilt
   `bzip2-1.0.8`, `m4-1.4.7`, `heirloom-devtools-070527`, `flex-2.5.11`,
   `flex-2.6.4`, `bison-2.3`, and `bison-3.4.1`.
5. Late utility and autotools ladder: `diffutils-2.7`, rebuilt `coreutils-5.0`,
   `coreutils-6.10`, `gawk-3.0.4`, `perl-5.000`, `perl-5.003`,
   `perl-5.004_05`, `perl-5.005_03`, `perl-5.6.2`, `autoconf-2.52`,
   `automake-1.6.3`, `autoconf-2.53`, `automake-1.7`, `autoconf-2.54`,
   `autoconf-2.55`, `automake-1.7.8`, `autoconf-2.57`, `autoconf-2.59`,
   `automake-1.8.5`, `autoconf-2.61`, `automake-1.9.6`,
   `automake-1.10.3`, `autoconf-2.64`, `automake-1.11.2`,
   `autoconf-2.69`, `libtool-2.2.4`, and `automake-1.15.1`.
6. Final `bootstrap/binutils-tcc.ncl`: binutils 2.30 with `as`, `ld`, `ar`,
   `ranlib`, `nm`, and `objcopy` installed for the gcc-4.0.4 transition.

**Rationale:** each epoch changes the available libc/build-tool surface.
Keeping epochs explicit makes host-leakage and linkage checks meaningful and
prevents a binutils-only placeholder replacement from skipping prerequisites.

### 2. Pin sources, patches, and generated artifacts at first consumption

**Choice:** each derivation declares its own `fetchTarball` or checked-in artifact
with URL/path, digest, first-consuming derivation, and provenance note rather
than sharing an implicit manifest-only source pool. Carried live-bootstrap patch
files and generated parser/lexer sources are stored under repo paths with BLAKE3
digests and upstream path/commit metadata.

**Rationale:** OpenSpec evidence and `scripts/check-bootstrap-source-pins.rs` can
then map a missing digest to the first consumer directly. It also prevents hidden
builder-script blobs from bypassing source review.

### 3. Fail closed on host leakage and legacy fallback

**Choice:** every epoch transcript records command, provider selection, exit
status, output path or failure class, fallback status/event marker, and
placeholder rejection result. The host-leakage audit scans command lines,
sandbox inputs, and proof markers for host compiler, host libc, host shell tool,
Nix command, and legacy-provider executable paths.

**Rationale:** the chain is only source-built if each stage uses prior chain
outputs. A single host shell/compiler fallback invalidates the binutils output
for parent full-source evidence.

## Risks / Trade-offs

**Large chain surface** → epoch boundaries keep commits and validation evidence
reviewable while preserving exact order.

**Patch provenance drift** → every carried patch or generated source must record
upstream path, upstream commit, BLAKE3 digest, and first consumer.

**Toolchain leakage** → no-host-leakage audit is a required validation task and
must fail on host compiler/libc/shell/Nix or legacy provider paths.

## Validation

Evidence must include exact command transcripts for each epoch. Every transcript
records command, provider selection, exit status, output path or failure class,
fallback status/event marker, and placeholder rejection result.

Required validation commands/checks:

- `./scripts/check-bootstrap-source-pins.rs bootstrap/*.ncl` for source, patch,
  generated-artifact, digest, and provenance coverage.
- Build each epoch derivation in dependency order with `crunch build <file>`.
- Run a no-host-leakage audit over each transcript.
- Run post-musl linkage checks proving `m4`, `flex`, `bison`, and `grep` link
  against musl.
- Run binutils assembler smoke: use produced `as` to assemble a trivial source
  into a valid ELF object.
