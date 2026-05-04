# I1 upstream ordering and output contract

Task-ID: I1
Covers: `bootstrap.part.bzip2.1.0.8.tcc`

## Upstream location

- Upstream repo: `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap`
- Part index: `parts.rst`, section `bzip2 1.0.8`.
- Step directory: `steps/bzip2-1.0.8/`.
- Source manifest: `steps/bzip2-1.0.8/sources`.

## Ordering

The first `bzip2 1.0.8` entry appears after the Mes-libc `sed 4.0.9` stage and before `coreutils 5.0`. A second `bzip2 1.0.8` entry appears later as the musl rebuild. This change is the TCC/Mes-libc part bound to `bootstrap/bzip2-tcc.ncl`, not the later musl rebuild.

Crunch keeps this part after the current TCC/Make/Patch bootstrap prerequisites via inputs in `bootstrap/bzip2-tcc.ncl`:

- `stage0-posix.ncl`
- `mes.ncl`
- `tinycc.ncl`
- `make-tcc.ncl`
- `patch-tcc.ncl`
- `bzip2-1.0.8-src`

## Source notes

Upstream `steps/bzip2-1.0.8/sources` pins:

```text
f https://sourceware.org/pub/bzip2/bzip2-1.0.8.tar.gz ab5a03176ee106d3f0fa90e381da478ddae405918153cca248e682cd0c4a2269
```

Crunch uses the same URL with a fixed `crunch.fetchTarball` SRI hash. That SRI hash is the Crunch/Nix-style fetched tree hash, not the upstream raw tarball checksum.

## Expected output contract

Upstream pass1 installs `bzip2` and `bunzip2` and smoke-runs `bzip2 --help`. Upstream pass2 installs `bzip2`, `bunzip2`, and `bzcat`.

Crunch's TCC derivation additionally builds and installs `bzip2recover`; therefore the local output contract is:

- `bin/bzip2` executable
- `bin/bunzip2` executable/symlink
- `bin/bzcat` executable/symlink
- `bin/bzip2recover` executable

The hardened derivation now checks all four installed paths and runs simple help/usage probes before success.
