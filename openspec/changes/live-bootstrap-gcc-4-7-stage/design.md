# Design: GCC 4.7.4 transition stage

## Context

`bootstrap/gcc-4.7.ncl` provides the C++11-capable compiler needed before modern
GCC. It consumes the gcc-4.0.4 stage and support tools from the source-built
chain.

## Decisions

### 1. Consume only gcc-4.0.4-era chain outputs

Allowed direct inputs for `bootstrap/gcc-4.7.ncl` are `bootstrap/gcc-4.0.ncl`,
`bootstrap/binutils-tcc.ncl`, the source-built musl/tcc outputs produced before
binutils 2.30, and the Perl/autoconf/automake/libtool/gawk/coreutils support
outputs already produced for the binutils-tcc chain. The derivation constructs
PATH only from those declared chain outputs and rejects `/usr`, Nix command,
legacy provider, or host compiler/libc/shell fallback markers. The accepted
marker is `fallback-event=none`; any `fallback-event=<kind>` other than `none`
blocks completion.

### 2. Pin concrete gcc 4.7.4 source and support artifacts

First consumer `bootstrap/gcc-4.7.ncl` records this artifact table:

| Artifact | URL/path | Digest | Provenance | First consumer |
|---|---|---|---|---|
| gcc-4.7.4 source | `https://ftpmirror.gnu.org/gcc/gcc-4.7.4/gcc-4.7.4.tar.bz2` | SHA-256 `92e61c6dc3a0a449e62d72a38185fda550168a86702dea07125ebd3ec3996282` | live-bootstrap `steps/gcc-4.7.4/sources` at commit `9a268c4c39cae952b268bc86da342be2175f03d4` | `bootstrap/gcc-4.7.ncl` |
| carried/generated support artifacts | none allowed in this change; if gcc-4.7.4 needs a carried/generated artifact, create a follow-up delta before implementation | n/a | not consumed | n/a |

### 3. Validate C++11 behavior

Validation compiles C, C++, and a minimal C++11 source with the produced compiler
and records transcript metadata, fallback status, and no-host audit result.

## Risks / Trade-offs

**Support-tool drift** → Any extra support tool pulled from live-bootstrap must
be pinned at first consumption.

**Host fallback** → configure scripts may find host tools; transcript checking is
required and blocks completion on fallback markers.

## Validation

Concrete validation commands:

- `./scripts/check-bootstrap-source-pins.rs bootstrap/gcc-4.7.ncl` writes
  `evidence/V1-source-pins.md`.
- `crunch build bootstrap/gcc-4.7.ncl` writes `evidence/V2-build.md`.
- `./scripts/check-bootstrap-transcript.rs --reject-host-tools evidence/V2-build.md`
  writes `evidence/V3-host-leakage.md`.
- C/C++/C++11 smoke commands compile with the produced gcc/g++ and write
  `evidence/V4-compiler-smoke.md`.

The transcript checker must verify declared-chain PATH construction, absence of
`/usr`/Nix/legacy-provider/host compiler/libc/shell paths, and
`fallback-event=none`.
