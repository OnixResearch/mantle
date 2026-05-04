# I2 derivation audit

Task-ID: I2
Covers: `bootstrap.part.bzip2.1.0.8.tcc`

## Audited file

- `bootstrap/bzip2-tcc.ncl`

## Upstream comparison

Upstream `steps/bzip2-1.0.8/pass1.kaem`:

- verifies the raw source tarball checksum through `checksum-transcriber` + `sha256sum -c`;
- extracts the gzip-compressed tarball;
- applies `patches/mes-libc.patch` and `patches/coreutils.patch`;
- builds with `make CC=tcc AR="tcc -ar" LDFLAGS="-static" bzip2`;
- installs `bzip2` and `bunzip2`;
- runs `bzip2 --help`;
- verifies `/usr/bin/bzip2` with `bzip2-1.0.8.checksums`.

Upstream `pass2.sh` rebuilds bzip2 later and installs `bzip2`, `bunzip2`, and `bzcat`.

## Crunch intentional deviations

- Crunch uses `crunch.fetchTarball` for the fixed source instead of replaying `checksum-transcriber` in the builder.
- Crunch manually compiles the bzip2 objects and archive with `tcc` rather than invoking the upstream Makefile. This keeps the derivation explicit and avoids relying on additional Makefile/autoconf behavior for a small fixed object set.
- Crunch currently does not apply the upstream Mes/coreutils patches. The manual compile path avoids the patched Makefile/install path; runtime validation still needs the actual bzip2 build/smoke before this can be considered fully proven.
- Crunch installs `bzip2recover` in addition to `bzip2`, `bunzip2`, and `bzcat`; the output contract must therefore check all four installed paths.
- Crunch keeps `make-tcc.ncl` and `patch-tcc.ncl` as predecessor inputs so this part remains ordered after the TCC-era toolchain boundary even though the manual compile path does not call `make` or `patch` directly.

## Audit result

The source-level contract is now self-contained for pins and installed outputs:

- source pin has provenance and first-consumer comments;
- required installed outputs fail closed (`bzip2`, `bunzip2`, `bzcat`, `bzip2recover`);
- simple help/usage probes execute for all installed commands.

Runtime completion remains assigned to V2/V3 after the long `crunch bootstrap validate bootstrap/bzip2-tcc.ncl` run finishes.
