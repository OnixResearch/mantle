# I1 upstream ordering and output contract

Task-ID: I1
Covers: `bootstrap.part.bzip2.1.0.8.musl`

## Upstream location

- Upstream repo: `/home/brittonr/git/pi-repos/fosslinux--live-bootstrap`
- Part index: `parts.rst`, second `bzip2 1.0.8` section in the musl rebuild group.
- Step directory: `steps/bzip2-1.0.8/`.

## Ordering

The musl rebuild entry appears after the first Mes/TCC `bzip2 1.0.8` part and after `sed 4.0.9` is rebuilt against musl. The upstream note says bzip2 is rebuilt unpatched with the new TCC and musl to fix stdin-reading issues from the previous build.

Crunch maps this part to `bootstrap/bzip2-1.0.8-musl.ncl` with direct inputs:

- `stage0-posix.ncl`
- `tcc-musl-v2.ncl`
- `musl-1.1.24-tcc-musl.ncl`
- `make-tcc.ncl`
- `bzip2-1.0.8-src`

## Source notes

Upstream `steps/bzip2-1.0.8/sources` pins the raw source tarball:

```text
f https://sourceware.org/pub/bzip2/bzip2-1.0.8.tar.gz ab5a03176ee106d3f0fa90e381da478ddae405918153cca248e682cd0c4a2269
```

Crunch uses the same URL with a fixed `crunch.fetchTarball` SRI tree hash.

## Expected output contract

Upstream musl pass2 installs `bzip2`, `bunzip2`, and `bzcat`. Crunch also builds and installs `bzip2recover`; therefore the local output contract is:

- `bin/bzip2` executable
- `bin/bunzip2` executable/symlink
- `bin/bzcat` executable/symlink
- `bin/bzip2recover` executable

The hardened derivation checks all four installed paths and runs simple help/usage probes before success.
